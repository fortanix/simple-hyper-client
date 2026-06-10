use http_body_util::BodyExt;
use hyper::body::{Body, Buf, Bytes};

/// Collects all of the data frames from this body into a [`Buf`].
///
/// This function avoids copying the data and is useful when you don't need
/// a contiguous slice of data.
pub async fn aggregate<T>(body: T) -> Result<impl Buf, T::Error>
where
    T: Body,
{
    Ok(body.collect().await?.aggregate())
}

/// Collects all of the data frames from this body into [`Bytes`].
pub async fn to_bytes<T>(body: T) -> Result<Bytes, T::Error>
where
    T: Body,
{
    Ok(body.collect().await?.to_bytes())
}
