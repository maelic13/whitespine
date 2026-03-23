use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chess::Board;
use tract_linalg::multithread::{Executor, multithread_tract_scope};
use tract_onnx::prelude::*;
use tract_onnx::tract_hir::infer::Factoid;
use tract_onnx::tract_hir::internal::DimLike;

use crate::neural_networks::{NetInputFactory, NetInputFn, NetInputVersion};

const CACHE_SIZE: usize = 100_000;

type RunnableModel = TypedRunnableModel<TypedModel>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeuralNetworkConfig {
    pub model_file: PathBuf,
    pub threads: usize,
}

pub struct NeuralNetworkEvaluator {
    model: RunnableModel,
    input_version: NetInputVersion,
    net_input: NetInputFn,
    executor: Executor,
    cache: RefCell<HashMap<u64, i32>>,
}

impl NeuralNetworkEvaluator {
    pub fn new(model_file: &Path, threads: usize) -> Result<NeuralNetworkEvaluator, String> {
        let threads = threads.max(1);

        let model = tract_onnx::onnx()
            .model_for_path(model_file)
            .map_err(|err| format!("Failed to load ONNX model '{}': {err}", model_file.display()))?;

        let input_version = NetInputVersion::from_planes(Self::read_input_planes(&model)?)?;
        let net_input = NetInputFactory::from_version(input_version);
        let model = model
            .into_optimized()
            .map_err(|err| format!("Failed to optimize ONNX model '{}': {err}", model_file.display()))?
            .into_runnable()
            .map_err(|err| format!("Failed to initialize ONNX runtime plan '{}': {err}", model_file.display()))?;

        Ok(NeuralNetworkEvaluator {
            model,
            input_version,
            net_input,
            executor: if threads == 1 {
                Executor::SingleThread
            } else {
                Executor::multithread(threads)
            },
            cache: RefCell::new(HashMap::new()),
        })
    }

    pub fn evaluate_board(&self, board: &Board) -> Result<i32, String> {
        let position_key = board.get_hash();
        if let Some(cached) = self.cache.borrow().get(&position_key).copied() {
            return Ok(cached);
        }

        let network_input = (self.net_input)(board);
        let input_tensor = Tensor::from_shape(
            &[1, self.input_version.planes(), 8, 8],
            &network_input,
        )
        .map_err(|err| format!("Failed to build ONNX input tensor: {err}"))?;

        let evaluation = multithread_tract_scope(self.executor.clone(), || {
            let outputs = self
                .model
                .run(tvec!(input_tensor.into_tvalue()))
                .map_err(|err| format!("ONNX inference failed: {err}"))?;
            let output = outputs
                .first()
                .ok_or_else(|| String::from("ONNX model returned no outputs."))?;
            let output_view = output
                .to_array_view::<f32>()
                .map_err(|err| format!("Failed to extract ONNX output tensor: {err}"))?;
            let raw_evaluation = output_view
                .iter()
                .copied()
                .next()
                .ok_or_else(|| String::from("ONNX output tensor is empty."))?;

            Ok::<i32, String>((raw_evaluation * 2000.0).round() as i32)
        })?;

        let mut cache = self.cache.borrow_mut();
        if cache.len() >= CACHE_SIZE {
            cache.clear();
        }
        cache.insert(position_key, evaluation);

        Ok(evaluation)
    }

    fn read_input_planes(model: &InferenceModel) -> Result<usize, String> {
        let input_fact = model
            .input_fact(0)
            .map_err(|err| format!("Failed to inspect ONNX model input: {err}"))?;
        let shape: Vec<_> = input_fact.shape.dims().collect();

        if shape.len() != 4 {
            return Err(format!(
                "Unsupported ONNX input rank: expected 4 dimensions, got {}.",
                shape.len()
            ));
        }

        shape[1]
            .concretize()
            .and_then(|dim: TDim| dim.to_usize().ok())
            .ok_or_else(|| String::from("ONNX model input channel dimension must be concrete."))
    }
}
