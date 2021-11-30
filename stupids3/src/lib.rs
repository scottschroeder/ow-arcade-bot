use failure::Fail;
use rusoto_core::{ByteStream, Region, RusotoError};
use rusoto_s3::{GetObjectError, GetObjectRequest, PutObjectRequest, S3Client, S3};
use serde::{de::DeserializeOwned, Serialize};
use std::io::Read;

#[derive(Debug, Fail)]
pub enum StupidS3Error {
    #[fail(display = "The key {}/{} does not exist", bucket, key)]
    NoSuchKey { bucket: String, key: String },
    #[fail(display = "The content was missing")]
    ContentMissing,
    #[fail(display = "The content was invalid")]
    ContentInvalid(#[fail(cause)] failure::Error),
    #[fail(
        display = "An unknown error occurred while retrieving {}/{} from S3",
        bucket, key
    )]
    UnknownError {
        bucket: String,
        key: String,
        #[fail(cause)]
        error: failure::Error,
    },
}

fn get_raw<B: AsRef<str>, K: AsRef<str>>(bucket: B, key: K) -> Result<ByteStream, StupidS3Error> {
    let s3_client = S3Client::new(Region::UsWest2);
    let resp = s3_client
        .get_object(GetObjectRequest {
            bucket: bucket.as_ref().into(),
            key: key.as_ref().into(),
            ..Default::default()
        })
        .sync()
        .map_err(|e| match e {
            RusotoError::Service(service_err) => match service_err {
                GetObjectError::NoSuchKey(_) => StupidS3Error::NoSuchKey {
                    bucket: bucket.as_ref().into(),
                    key: key.as_ref().into(),
                },
            },
            _ => StupidS3Error::UnknownError {
                bucket: bucket.as_ref().into(),
                key: key.as_ref().into(),
                error: e.into(),
            },
        })?;
    let bytes = resp.body.ok_or(StupidS3Error::ContentMissing)?;
    Ok(bytes)
}

pub fn get<B: AsRef<str>, K: AsRef<str>>(bucket: B, key: K) -> Result<String, StupidS3Error> {
    let bytes = get_raw(bucket, key)?;
    let mut buf = String::new();
    bytes
        .into_blocking_read()
        .read_to_string(&mut buf)
        .map_err(|e| StupidS3Error::ContentInvalid(e.into()))?;
    Ok(buf)
}

pub fn get_obj<S: DeserializeOwned, B: AsRef<str>, K: AsRef<str>>(
    bucket: B,
    key: K,
) -> Result<S, StupidS3Error> {
    let bytes = get_raw(bucket, key)?;
    let obj: S = serde_json::from_reader(bytes.into_blocking_read())
        .map_err(|e| StupidS3Error::ContentInvalid(e.into()))?;
    Ok(obj)
}

pub fn put<S: Serialize, B: AsRef<str>, K: AsRef<str>>(
    bucket: B,
    key: K,
    obj: &S,
) -> Result<(), failure::Error> {
    let s3_client = S3Client::new(Region::UsWest2);
    let _resp = s3_client
        .put_object(PutObjectRequest {
            bucket: bucket.as_ref().into(),
            key: key.as_ref().into(),
            body: Some(serde_json::to_string_pretty(obj)?.into_bytes().into()),
            ..Default::default()
        })
        .sync()?;
    Ok(())
}
