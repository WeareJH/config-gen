use actix_web::client::ClientRequest;
use actix_web::client::ClientResponse;
use actix_web::test;
use actix_web::test::TestApp;
use actix_web::test::TestServer;
use actix_web::HttpMessage;
use app_state::AppState;
use std::str;

///
/// Helper to create a test server with a handler
/// that returns the server + it's address
///
pub fn get_test_server<H>(handler: H) -> (TestServer, String)
where
    H: Fn(&mut TestApp) + Send + Sync + Clone + 'static,
{
    let target = test::TestServer::new(handler);
    let target_addr = target.addr().to_string();
    (target, target_addr)
}

///
/// Helper to create the proxy test server + it's address
///
pub fn get_test_proxy<H>(server: &TestServer, handler: H) -> (TestServer, String)
where
    H: Fn(&mut TestApp<AppState>) + Send + Sync + Clone + 'static,
{
    let srv_address = server.addr().to_string();
    let p = test::TestServer::build_with_state(move || {
        let addr = srv_address.clone();
        let s = AppState::new(addr.clone(), "http");
        s
    })
    .start(handler);

    let p_addr = p.addr().to_string();
    (p, p_addr)
}

///
/// Helper to get both the ClientResponse + body as a string
///
pub fn get_resp(proxy: &mut TestServer, req: ClientRequest) -> (ClientResponse, String) {
    let response = proxy.execute(req.send()).expect("proxy executed the send");
    let _bytes = proxy
        .execute(response.body())
        .expect("got proxy response body");
    let response_body = str::from_utf8(&_bytes[..])
        .expect("bytes->String")
        .to_string();
    (response, response_body)
}
