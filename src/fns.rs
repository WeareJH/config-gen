use actix_web::{
    client, AsyncResponder, Body, Error, HttpMessage,
    HttpRequest, HttpResponse, http, dev
};
use actix::run;
use actix_web::http::header;
use futures::{Future, Stream};
use futures::future::{Either, ok};
use rewrites::replace_host;
use options::ProxyOpts;
use std::str;
use actix_web::http::HeaderMap;
use regex::Regex;
use regex::Captures;
use http::header::HeaderValue;
use actix_web::client::ClientRequest;
use actix_web::http::Cookie;

///
/// This function will clone incoming requests
/// and pass them onto a backend specified via the `target` field on [ProxyOpts]
///
pub fn proxy_transform(_req: &HttpRequest<ProxyOpts>) -> Box<Future<Item=HttpResponse, Error=Error>> {

    // building up the new request that we'll send to the backend
//    let mut outgoing = client::ClientRequest::build_from(_req);
    let req_headers = _req.headers().clone();
    let next_host = _req.uri().clone();
    let req_host = next_host.host().unwrap_or("");
    let req_port = next_host.port().unwrap_or(80);
    let req_target = format!("{}:{}", req_host, req_port);
    let cloned = clone_headers(&req_headers, req_target, _req.state().target.clone());

    // build up the next outgoing URL (for the back-end)
    let next_url = format!("{}://{}{}{}",
                           match _req.uri().scheme_part() {
                               Some(scheme) => scheme.as_str(),
                               None => "http"
                           },
                           _req.state().target.clone(),
                           _req.path(),
                           match _req.uri().query().as_ref() {
                               Some(q) => format!("?{}", q),
                               None => "".to_string()
                           });

    // now choose how to handle it
    // if the client responds with a request we want to alter (such as HTML)
    // then we need to buffer the body into memory in order to apply regex's on the string
    let next_target = _req.state().target.clone();
    let next_host = _req.uri().clone();
    let original_method = _req.method().as_str().clone();

    let mut outgoing = client::ClientRequest::build();
    outgoing.method(_req.method().clone()).uri(next_url);

    for (key, value) in cloned.iter() {
        outgoing.header(key.clone(), value.clone());
    }


    if original_method == "POST" {
        println!("POST SEND-> {:?}", outgoing);
        let outgoing = _req.body()
            .from_err()
            .and_then(move |incoming_body| {
                println!("POST SEND BODY-> {:?}", incoming_body);
                outgoing.body(incoming_body).unwrap().send().map_err(Error::from)
                    .and_then(move |proxy_response| {
                        println!("POST resp from proxy {:?}", &proxy_response);
                        let req_host = next_host.host().unwrap_or("");
                        let req_port = next_host.port().unwrap_or(80);
                        let req_target = format!("{}:{}", req_host, req_host);
                        proxy_response.body()
                            .from_err()
                            .and_then(move |proxy_response_body| {
                                Ok(create_outgoing(
                                    &proxy_response.headers(),
                                    next_target.to_string(),
                                    req_target
                                ).body(proxy_response_body))
                            })
                    })
            });

        Box::new(outgoing)
    } else {
        outgoing.finish().unwrap().send().map_err(Error::from)
            .and_then(move |proxy_response| {

//                println!("GET resp from proxy {:?}", &proxy_response);
                // Should we rewrite this response?
                // just check for the correct content-type header for now.
                // This will need fleshing out to provide stricter checks
                let rewrite_response = match proxy_response.headers().get(header::CONTENT_TYPE) {
                    Some(t) => {
                        match t.to_str().unwrap_or("") {
                            "text/html" | "text/html; charset=UTF-8" => true,
                            _ => false,
                        }
                    }
                    _ => false
                };

                // If we decide to modify the response, we need to buffer the entire
                // response into memory (text files only)
                if rewrite_response {
                    Either::A(
                        proxy_response.body()
                            .from_err()
                            .and_then(move |body| {
                                use std::str;

                                // In here, we now have a ful buffered response body
                                // so we can go ahead and apply URL replacements
                                let req_host = next_host.host().unwrap_or("");
                                let req_port = next_host.port().unwrap_or(80);
                                let req_target = format!("{}:{}", req_host, req_host);
                                let next_body = replace_host(
                                    str::from_utf8(&body[..]).unwrap(),
                                    &next_target,
                                    req_host, req_port,
                                );
                                let as_string = next_body.to_string();
                                Ok(create_outgoing(&proxy_response.headers(), next_target.to_string(), req_target).body(as_string))
                            })
                    )
                } else {
                    let req_host = next_host.host().unwrap_or("");
                    let req_port = next_host.port().unwrap_or(80);
                    let req_target = format!("{}:{}", req_host, req_host);
                    // If we get here, we decided not to re-write the response
                    // so we just stream it back to the client
                    Either::B(
                        ok(create_outgoing(&proxy_response.headers(), next_target.to_string(), req_target).body(Body::Streaming(Box::new(proxy_response.payload().from_err()))))
                    )
                }
            })
            .responder()
    }
}

fn create_outgoing(resp_headers: &HeaderMap, target: String, replacer: String) -> dev::HttpResponseBuilder {
    let mut outgoing = HttpResponse::Ok();
    let c = clone_headers(resp_headers, target, replacer);
    // Copy headers from backend response to main response
    for (key, value) in c.iter() {
        outgoing.header(key.clone(), value.clone());
    }
    outgoing
}

fn clone_headers(headers: &HeaderMap, target: String, replacer: String) -> HeaderMap {
    println!("matching = {}", target);
    let regex = Regex::new(target.as_str()).unwrap();
    let mut hm = HeaderMap::new();
    for (key, value) in headers.iter() {
        let strs = value.to_str().unwrap();

        let next_string = match key.as_str() {
            "set-cookie" => {
                let mut c = Cookie::parse_encoded(strs).unwrap();
                c.set_domain("");
                c.to_string()
            },
            _ => strs.to_string()
        };

        let next = regex.replace(&next_string, replacer.as_str());
        let hv = HeaderValue::from_str(&next);
        hm.append(key.clone(), hv.unwrap());
    }
    hm
}

#[test]
pub fn test_clone_headers() {
    let mut hm = HeaderMap::new();
    hm.append("none-dup", "form_key=123456".parse().unwrap());
    hm.append("set-cookie", "form_key=123456; domain=www.neom.com".parse().unwrap());
    hm.append("set-cookie", "key=value; domain=www.neom.com".parse().unwrap());

    // cloned header map with domain re - written
    let cloned = clone_headers(&hm, "www.neom.com".to_string(), "127.0.0.1:8080".to_string());

//    println!("{:#?}", cloned);

    // expected header map
    let mut expected = HeaderMap::new();
    expected.append("none-dup", "form_key=123456".parse().unwrap());
    expected.append("set-cookie", "form_key=123456; Domain=".parse().unwrap());
    expected.append("set-cookie", "key=value; Domain=".parse().unwrap());

    assert_eq!(expected, cloned);
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;
    use mime::TEXT_HTML;
    use actix_web::http::Cookie;
    use actix_web::http::Method;

    const STR: &str = "Hello world";

    #[test]
    fn test_forwards_headers() {
        let server = test::TestServer::new(|app| {
            app.handler(|req: &HttpRequest| {
                println!("headers received at proxy addr: {:#?}", req.headers());
                assert_eq!(req.headers().get(header::ACCEPT).unwrap(), "text/html");
                HttpResponse::Ok()
                    .header("shane", "kittens")
                    .header(header::CONTENT_TYPE, TEXT_HTML)
                    .body(STR)
            })
        });

        let srv_address = server.addr().to_string();
        let srv_address2 = server.addr().to_string();

        let mut proxy = test::TestServer::build_with_state(move || {
            let addr = srv_address.clone();
            ProxyOpts::new(addr.clone())
        })
            .start(move |app| {
                app.handler(proxy_transform);
            });

        let request = proxy.get()
            .header(header::ACCEPT, "text/html")
            .set_header(header::ORIGIN, format!("https://{}", proxy.addr().to_string()))
            .uri(proxy.url("/"))
            .finish()
            .unwrap();

        let response = proxy.execute(request.send()).unwrap();
        let _bytes = proxy.execute(response.body()).unwrap();

        println!("main resp: {:#?}", response.headers());
        println!("bytes={:#?}", _bytes);

        let has_header = response.headers().get("shane").is_some();

        assert_eq!(has_header, true);
    }

    #[test]
    fn test_forwards_post_requests() {
        use bytes::Bytes;

        let server = test::TestServer::new(|app| {
            app.handler(|req: &HttpRequest| {
                println!("headers received at proxy addr: {:#?}", req.headers());
                println!("method received at proxy: {:#?}", req.method());
                assert_eq!(req.headers().get(header::ACCEPT).unwrap(), "text/html");
                req.body()
                    .and_then(move |bytes: Bytes| {
                        Ok(
                            HttpResponse::Ok()
                                .header(header::CONTENT_TYPE, "application/json")
                                .header(header::SET_COOKIE, "form_key=40je6TqaB2SDRBeV; expires=Thu, 09-Aug-2018 10:23:41 GMT; Max-Age=10800; path=/; domain=www.neomorganics.com")
                                .body(format!("REC-->{}", str::from_utf8(&bytes[..]).unwrap().to_string()))
                        )
                    })
                    .responder()
            })
        });

        let srv_address = server.addr().to_string();
        let srv_address2 = server.addr().to_string();

        let mut proxy = test::TestServer::build_with_state(move || {
            let addr = srv_address.clone();
            ProxyOpts::new(addr.clone())
        })
            .start(move |app| {
                app.handler(proxy_transform);
            });

        let request = proxy.post()
            .uri(proxy.url("/"))
            .header(header::ACCEPT, "text/html")
            .body(r#"{"hello": "world"}"#)
            .unwrap();

        let response = proxy.execute(request.send()).unwrap();
        let _bytes = proxy.execute(response.body()).unwrap();

        println!("main resp: {:#?}", response.headers());
        println!("bytes={:#?}", _bytes);

        assert_eq!(_bytes, r#"REC-->{"hello": "world"}"#)
    }

    #[test]
    fn test_strip_domain_from_cookies() {
        let cookie_value = "form_key=40je6TqaB2SDRBeV; expires=Thu, 09-Aug-2018 10:23:41 GMT; Max-Age=10800; path=/; domain=www.neomorganics.com";
        let cookie = Cookie::build("form_key", "40je6TqaB2SDRBeV")
            .domain("www.neomorganics.com")
            .finish();
        println!("{}", cookie);
        let mut parsed = Cookie::parse(cookie_value).unwrap();
        parsed.set_domain("");
        println!("{}", parsed.to_string());
    }

    #[test]
    fn test_works() {
        run(|| {
            ClientRequest::build()
                .method(Method::POST)
                .uri("https://www.neomorganics.com/checkout/cart/add/uenc/aHR0cHM6Ly93d3cubmVvbW9yZ2FuaWNzLmNvbS9uYXYvc2tpbmNhcmUuaHRtbA%2C%2C/product/653/")
                .header("Content-Type", "application/x-www-form-urlencoded; charset=UTF-8")
                .header("X-Requested-With", "XMLHttpRequest")
                .header("Cookie", "\
                store=default; \
                PHPSESSID=bgj1kg7nvjh9pie4rc7o1lnm50; \
                private_content_version=02ab871ca8023271a480bda1390ae395; \
                login_redirect=https://www.neomorganics.com/nav/skincare.html; \
                ometria=2_cid%3DSslvCsRfe41DlMYO%26nses%3D1%26osts%3D1533975448%26sid%3D34fb0e56q7CB70wjbnzJ%26npv%3D1%26slt%3D1533975448; \
                2c.cId=5b6e9b9860b2ff13497f14b9; \
                _hc_exp={*_cr*!1533975448526}; \
                cookie-notice=agreed; \
                mage-cache-storage=%7B%7D; \
                mage-cache-storage-section-invalidation=%7B%7D; \
                _hjIncludedInSample=1; \
                mage-cache-sessid=true; \
                mage-banners-cache-storage=%7B%7D; \
                form_key=hJdzEzdwJdcaqXjy; \
                mage-messages=; \
                2c.dc=%7B%225b2a291e60b2572bb13baee2%22%3A%7B%22state%22%3A%22closed%22%2C%22coupon%22%3Anull%7D%7D; \
                _ga=GA1.2.515757602.1533975454; \
                _gid=GA1.2.202401003.1533975454; \
                _gat_UA-8638482-7=1; \
                __qca=P0-846161877-1533975453676; \
                recently_viewed_product=%7B%7D; \
                recently_viewed_product_previous=%7B%7D; \
                recently_compared_product=%7B%7D; \
                recently_compared_product_previous=%7B%7D; \
                product_data_storage=%7B%7D; \
                _hc_cart=1085458958; \
                section_data_ids=%7B%22cart%22%3A1533975455%2C%22hic-cart-data%22%3A1533975453%2C%22cart-tagging%22%3A1533975453%2C%22directory-data%22%3A1533975453%2C%22customer%22%3A1533975453%2C%22compare-products%22%3A1533975453%2C%22last-ordered-items%22%3A1533975453%2C%22wishlist%22%3A1533975453%2C%22hic-user-data%22%3A1533975453%2C%22instant-purchase%22%3A1533975453%2C%22multiplewishlist%22%3A1533975453%2C%22review%22%3A1533975453%2C%22extra-information%22%3A1533975453%2C%22customer-tagging%22%3A1533975453%2C%22recently_viewed_product%22%3A1533975453%2C%22recently_compared_product%22%3A1533975453%2C%22product_data_storage%22%3A1533975453%2C%22paypal-billing-agreement%22%3A1533975453%2C%22checkout-fields%22%3A1533975453%2C%22collection-point-result%22%3A1533975453%7D; \
                _hc_vid={*id*!*99f3ab4a-f877-415d-b6ce-41f58cb27fcb*~*created*!1533975176036~*psq*!10~*ord*!29~*cl*!7~*gbl*!0}; \
                _hc_ses={*id*!*1946be54-8146-46b6-8833-71a9abd54d42*~*created*!1533975176038~*isNew*!true~*psq*!10~*ord*!29~*cl*!7~*ser*!false~*attr*![*(direct)*~*direct*~*(not+set)*~*(not+set)*~*(none)*~*(direct)*]~*ap*!*home*}")
                .body("product=653&uenc=aHR0cHM6Ly93d3cubmVvbW9yZ2FuaWNzLmNvbS9jaGVja291dC9jYXJ0L2FkZC91ZW5jL2FIUjBjSE02THk5M2QzY3VibVZ2Ylc5eVoyRnVhV056TG1OdmJTOXVZWFl2YzJ0cGJtTmhjbVV1YUhSdGJBJTJDJTJDL3Byb2R1Y3QvNjUzLw%2C%2C&form_key=hJdzEzdwJdcaqXjy")
                    .unwrap()
                    .send()
                    .map_err(|_| ())
                    .and_then(|resp| {
                        println!("res[] {:?}", resp);
                        Ok(())
                    })
        });
    }
}
