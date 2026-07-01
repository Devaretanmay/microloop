use anyhow::{Error as E, Result};
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config, DTYPE};
use hf_hub::{api::sync::Api, Repo, RepoType};
use tokenizers::{PaddingParams, Tokenizer};

pub struct SemanticModel {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
}

impl SemanticModel {
    pub fn new() -> Result<Self> {
        let device = Device::Cpu;
        let repo = Repo::with_revision(
            "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            RepoType::Model,
            "refs/pr/21".to_string(),
        );
        
        println!("Loading model from Hugging Face Hub (this may download weights on first run)...");
        let api = Api::new()?;
        let api = api.repo(repo);
        
        let config_filename = api.get("config.json")?;
        let tokenizer_filename = api.get("tokenizer.json")?;
        let weights_filename = api.get("model.safetensors")?;

        let config = std::fs::read_to_string(config_filename)?;
        let config: Config = serde_json::from_str(&config)?;
        
        let mut tokenizer = Tokenizer::from_file(tokenizer_filename)
            .map_err(|e| E::msg(e.to_string()))?;
        
        if let Some(pp) = tokenizer.get_padding_mut() {
            pp.strategy = tokenizers::PaddingStrategy::BatchLongest;
        } else {
            let pp = PaddingParams {
                strategy: tokenizers::PaddingStrategy::BatchLongest,
                ..Default::default()
            };
            tokenizer.with_padding(Some(pp));
        }

        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[weights_filename], DTYPE, &device)? };
        let model = BertModel::load(vb, &config)?;

        Ok(Self {
            model,
            tokenizer,
            device,
        })
    }

    pub fn embed(&self, texts: &[&str]) -> Result<Tensor> {
        let tokens = self
            .tokenizer
            .encode_batch(texts.to_vec(), true)
            .map_err(|e| E::msg(e.to_string()))?;
        
        let token_ids: Vec<u32> = tokens.iter().flat_map(|t| t.get_ids().to_vec()).collect();
        let token_ids = Tensor::from_vec(
            token_ids,
            (texts.len(), tokens[0].get_ids().len()),
            &self.device,
        )?;
        
        let token_type_ids = token_ids.zeros_like()?;
        let embeddings = self.model.forward(&token_ids, &token_type_ids, None)?;
        
        let (_n_sentence, n_tokens, _hidden_size) = embeddings.dims3()?;
        let embeddings = (embeddings.sum(1)? / (n_tokens as f64))?;
        
        let norm = embeddings.sqr()?.sum_keepdim(1)?.sqrt()?;
        let normalized = embeddings.broadcast_div(&norm)?;
        
        Ok(normalized)
    }
}

pub fn cosine_similarity(a: &Tensor, b: &Tensor) -> Result<f32> {
    let dot = (a * b)?.sum_all()?;
    let val: f32 = dot.to_scalar()?;
    Ok(val)
}
