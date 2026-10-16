use tikus_api::{Result, TikusClient};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Tikus Rust Client: Authentication Demo ===");
    let mut client = TikusClient::new();

    // 1. Request SMS OTP
    let country_code = "+251";
    let phone_number = "911223344";

    println!("[*] Sending OTP to {} {}...", country_code, phone_number);
    match client.send_otp(country_code, phone_number).await {
        Ok(resp) => println!("[+] OTP request response: status={:?}", resp.status),
        Err(err) => println!("[-] Send OTP notice: {}", err),
    }

    // 2. Demonstrate Google Native OAuth login
    let dummy_google_id_token = "eyJhbGciOiJSUzI1NiIsImtpZCI6IjEyMyJ9...";
    println!("\n[*] Testing Google OAuth native login format...");
    match client.login_with_google(dummy_google_id_token).await {
        Ok(auth) => {
            println!("[+] Successfully logged in!");
            println!("    Token: {:?}", auth.token);
            if let Some(user) = auth.user {
                println!("    User: {}", user.display_name());
            }
        }
        Err(err) => println!("[-] Google login response (expected without real token): {}", err),
    }

    Ok(())
}
