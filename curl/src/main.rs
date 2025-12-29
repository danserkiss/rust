use curl::curl::CURL;

fn main() {
    let mut curl = CURL::new();
    let mut resp = String::new();

    let url = "https://stackoverflow.com/questions";
    // let url = "127.0.0.1:3030/users";

    match curl.set_url(url) {
        Ok(_) => {}
        Err(_) => {
            println!("Error setting URL");
        }
    }

    match curl.set_write_to_string(&mut resp) {
        Ok(_) => {}
        Err(_) => {
            println!("Error setting URL");
        }
    }

    println!("result: {:?}", curl.perform());
    println!("Response ({} bytes)\n{}", resp.len(), resp);
}
