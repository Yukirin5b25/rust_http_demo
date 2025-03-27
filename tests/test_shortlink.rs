use rust_http_demo::handlers::generate_shortlink;

#[test]
fn test_generate_shortlink() {
    let url = "https://example.com";
    let short = generate_shortlink(url, None, Some(8));
    assert_eq!(short, "6JvlOnj0");

    let identifier = "example";
    let short = generate_shortlink(url, Some(identifier), Some(8));
    assert_eq!(short, "6Xldem53");
}
