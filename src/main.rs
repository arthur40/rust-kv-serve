mod request;

use request::Request;

fn main() {
    let mut request = Request::new(1, 5);

    while !request.is_finished() {
        request.step();

        println!(
            "Request {}: token {}",
            request.id,
            request.generated_tokens()
        );
    }

    println!("Request {} finished", request.id);
}
