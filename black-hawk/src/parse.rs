use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    sequence::tuple,
    IResult,
};

use crate::{
    consts::{ContentType, TransferEncoding},
    error::RequestParseError,
    request::{
        boundary::{self, Boundary},
        path::HttpPath,
        HttpHeaders, HttpMethod, HttpRequestHeader, HttpVersion, RequestBody,
    },
};

pub(crate) fn parse_method(i: &[u8]) -> IResult<&[u8], HttpMethod> {
    let (i, method_str) = alt((
        tag("GET"),
        tag("POST"),
        tag("PUT"),
        tag("OPTIONS"),
        tag("DELETE"),
        tag("PATCH"),
        tag("CONNECT"),
        tag("HEAD"),
        tag("TRACE"),
    ))(i)?;
    let method = match method_str {
        b"GET" => HttpMethod::Get,
        b"POST" => HttpMethod::Post,
        b"PUT" => HttpMethod::Put,
        b"OPTIONS" => HttpMethod::Options,
        b"DELETE" => HttpMethod::Delete,
        b"PATCH" => HttpMethod::Patch,
        b"CONNECT" => HttpMethod::Connect,
        b"HEAD" => HttpMethod::Head,
        b"TRACE" => HttpMethod::Trace,
        _ => unreachable!("never get here!"),
    };
    Ok((i, method))
}

pub(crate) fn parse_white_space(i: &[u8]) -> IResult<&[u8], ()> {
    Ok((tag(" ")(i)?.0, ()))
}

pub(crate) fn parse_path(i: &[u8]) -> Result<(&[u8], HttpPath), RequestParseError> {
    let (i, path_str) = take_until::<_, _, nom::error::Error<&[u8]>>(" ")(i)?;
    Ok((i, HttpPath::from_request(path_str)?))
}

pub(crate) fn parse_version(i: &[u8]) -> IResult<&[u8], HttpVersion> {
    let (i, (_, version_str)) = tuple((
        tag("HTTP/"),
        alt((tag("1.0"), tag("1.1"), tag("2"), tag("3"))),
    ))(i)?;
    let version = match version_str {
        b"1.0" => HttpVersion::V1_0,
        b"1.1" => HttpVersion::V1_1,
        b"2" => HttpVersion::V2,
        b"3" => HttpVersion::V3,
        _ => unreachable!("never get here!"),
    };
    Ok((i, version))
}

pub(crate) fn parse_new_line(i: &[u8]) -> IResult<&[u8], ()> {
    Ok((tag("\r\n")(i)?.0, ()))
}

pub(crate) fn parse_header_pair(i: &[u8]) -> IResult<&[u8], (String, String)> {
    let (i, (name_str, _)) = tuple((take_until(":"), tag(": ")))(i)?;
    let value_str = i;
    Ok((
        i,
        (
            String::from_utf8_lossy(name_str).to_string(),
            String::from_utf8_lossy(value_str).to_string(),
        ),
    ))
}

pub(crate) fn parse_http_header(i: &[u8]) -> Result<HttpRequestHeader, RequestParseError> {
    let (i, method) = parse_method(i)?;
    let (i, _) = parse_white_space(i)?;
    let (i, path) = parse_path(i)?;
    let (i, _) = parse_white_space(i)?;
    let (i, version) = parse_version(i)?;
    let (i, _) = parse_new_line(i)?;
    let mut headers = HttpHeaders(HashMap::new());
    let header_lines = String::from_utf8_lossy(i);
    for line in header_lines.lines() {
        dbg!(line);
        if line.trim().is_empty() {
            continue;
        }
        let (_, (name, value)) = parse_header_pair(line.trim().as_bytes())?;
        headers.0.insert(name, value);
    }
    Ok(HttpRequestHeader {
        method,
        path,
        version,
        headers,
    })
}

pub(crate) fn parse_chunked_body(i: &[u8]) -> Result<RequestBody, RequestParseError> {
    let chunked_lines = String::from_utf8_lossy(i);
    let mut all_contents = String::new();
    let mut chunk_sizes = vec![];
    let mut c_lines = chunked_lines.lines();
    while let Some(line) = c_lines.next() {
        if line.trim().is_empty() {
            continue;
        }
        let chunk_size = usize::from_str_radix(line.trim(), 16)?;
        if chunk_size == 0 {
            break;
        }
        let line = c_lines.next().ok_or(RequestParseError::ParseChunkContent)?;
        if line.len() != chunk_size {
            return Err(RequestParseError::ChunkContentLengthUnmatch(
                chunk_size,
                line.len(),
            ));
        }
        chunk_sizes.push(chunk_size);
        all_contents.push_str(line);
    }
    Ok(RequestBody::Chunked {
        content: all_contents,
        sizes: chunk_sizes,
    })
}

pub(crate) fn parse_multipart_boundary(body: &[u8], boundary: &str) -> Vec<Boundary> {
    todo!()
}

pub(crate) fn parse_request_body(
    body: &[u8],
    header: &HttpRequestHeader,
) -> anyhow::Result<RequestBody> {
    if let Some(TransferEncoding::Chunked) = header.transfer_encoding() {
        return Ok(parse_chunked_body(body)?);
    }
    match header.content_type() {
        Some(ContentType::ApplicationJson) => {
            let j = serde_json::from_slice::<serde_json::Value>(body)?;
            Ok(RequestBody::Json(j))
        }
        Some(ContentType::MultiPart(boundary)) => Ok(RequestBody::MultiPart(
            parse_multipart_boundary(body, &boundary),
        )),
        None => Ok(RequestBody::RawText(
            String::from_utf8_lossy(body).to_string(),
        )),
        _ => Err(RequestParseError::UnknownContentType.into()),
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_curl_get_parse() {
        let raw = "GET /test HTTP/1.1\r\nUser-Agent: curl/7.18.0 (i486-pc-linux-gnu) libcurl/7.18.0 OpenSSL/0.9.8g zlib/1.2.3.3 libidn/1.1\r\nHost: 0.0.0.0=5000\r\nAccept: */*\r\n\r\n";
        let (i, method) = parse_method(raw.as_bytes()).unwrap();
        assert_eq!(method, HttpMethod::Get);
        let (i, _) = parse_white_space(i).unwrap();
        let (i, path) = parse_path(i).unwrap();
        assert_eq!(path.abs_path(), "/test");
        let (i, _) = parse_white_space(i).unwrap();
        let (i, version) = parse_version(i).unwrap();
        assert_eq!(version, HttpVersion::V1_1);
        let (_i, _) = parse_new_line(i).unwrap();
        let http_header = parse_http_header(raw.as_bytes()).unwrap();
        assert_eq!(http_header.method, HttpMethod::Get);
        assert_eq!(http_header.version, HttpVersion::V1_1);
        assert_eq!(http_header.path.abs_path(), "/test");
        assert_eq!(
            http_header.headers.0.get("Accept"),
            Some(&"*/*".to_string())
        );
        assert_eq!(
            http_header.headers.0.get("Host"),
            Some(&"0.0.0.0=5000".to_string())
        );
        assert_eq!(http_header.headers.0.get("User-Agent"), Some(&"curl/7.18.0 (i486-pc-linux-gnu) libcurl/7.18.0 OpenSSL/0.9.8g zlib/1.2.3.3 libidn/1.1".to_string()));
    }

    #[test]
    fn test_firefox_get_test() {
        let raw = "GET /favicon.ico HTTP/1.1\r\nHost: 0.0.0.0=5000\r\nUser-Agent: Mozilla/5.0 (X11; U; Linux i686; en-US; rv:1.9) Gecko/2008061015 Firefox/3.0\r\nAccept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\nAccept-Language: en-us,en;q=0.5\r\nAccept-Encoding: gzip,deflate\r\nAccept-Charset: ISO-8859-1,utf-8;q=0.7,*;q=0.7\r\nKeep-Alive: 300\r\nConnection: keep-alive\r\n\r\n";
        let http_header = parse_http_header(raw.as_bytes()).unwrap();
        assert_eq!(http_header.method, HttpMethod::Get);
        assert_eq!(http_header.path.abs_path(), "/favicon.ico");
        assert_eq!(http_header.version, HttpVersion::V1_1);
        assert_eq!(http_header.headers.0.get("Host").unwrap(), "0.0.0.0=5000");
        assert_eq!(
            http_header.headers.0.get("User-Agent").unwrap(),
            "Mozilla/5.0 (X11; U; Linux i686; en-US; rv:1.9) Gecko/2008061015 Firefox/3.0"
        );
        assert_eq!(
            http_header.headers.0.get("Accept").unwrap(),
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
        );
        assert_eq!(
            http_header.headers.0.get("Accept-Language").unwrap(),
            "en-us,en;q=0.5"
        );
        assert_eq!(
            http_header.headers.0.get("Accept-Encoding").unwrap(),
            "gzip,deflate"
        );
        assert_eq!(
            http_header.headers.0.get("Accept-Charset").unwrap(),
            "ISO-8859-1,utf-8;q=0.7,*;q=0.7"
        );
        assert_eq!(http_header.headers.0.get("Keep-Alive").unwrap(), "300");
        assert_eq!(
            http_header.headers.0.get("Connection").unwrap(),
            "keep-alive"
        );
    }
}
