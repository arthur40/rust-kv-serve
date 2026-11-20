use crate::request::Request;

pub trait Backend {
    fn generate_batch(&mut self, requests: &[Request]) -> Vec<String>;
}

pub struct SimulatedBackend;

impl Backend for SimulatedBackend {
    fn generate_batch(&mut self, requests: &[Request]) -> Vec<String> {
        requests
            .iter()
            .map(|request| format!("token_{}", request.generated_tokens() + 1))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{Backend, SimulatedBackend};
    use crate::request::Request;

    #[test]
    fn batch_tokens_follow_each_requests_progress() {
        let mut backend = SimulatedBackend;
        let mut first = Request::new(1, 3);
        let second = Request::new(2, 3);

        first.step();

        let requests = vec![first, second];
        let tokens = backend.generate_batch(&requests);

        assert_eq!(tokens, vec!["token_2", "token_1"]);
        assert_eq!(requests[0].generated_tokens(), 1);
        assert_eq!(requests[1].generated_tokens(), 0);
    }

    #[test]
    fn empty_batch_produces_no_tokens() {
        let mut backend = SimulatedBackend;

        assert!(backend.generate_batch(&[]).is_empty());
    }
}
