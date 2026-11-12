pub struct Request {
    pub id: u64,
    max_tokens: usize,
    generated_tokens: usize,
}

impl Request {
    pub fn new(id: u64, max_tokens: usize) -> Self {
        Self {
            id,
            max_tokens,
            generated_tokens: 0,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.generated_tokens >= self.max_tokens
    }

    pub fn step(&mut self) {
        if !self.is_finished() {
            self.generated_tokens += 1;
        }
    }

    pub fn generated_tokens(&self) -> usize {
        self.generated_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::Request;

    #[test]
    fn request_finishes_after_max_tokens() {
        let mut request = Request::new(1, 2);

        assert_eq!(request.generated_tokens(), 0);
        assert!(!request.is_finished());

        request.step();
        assert_eq!(request.generated_tokens(), 1);
        assert!(!request.is_finished());

        request.step();
        assert_eq!(request.generated_tokens(), 2);
        assert!(request.is_finished());

        request.step();
        assert_eq!(request.generated_tokens(), 2);
    }

    #[test]
    fn zero_token_request_is_already_finished() {
        let mut request = Request::new(1, 0);

        assert!(request.is_finished());

        request.step();
        assert_eq!(request.generated_tokens(), 0);
    }
}
