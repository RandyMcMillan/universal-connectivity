use async_trait::async_trait;
use futures::{io, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use libp2p::{request_response, StreamProtocol};
use serde::{Deserialize, Serialize};

// Constants for maximum data transfer sizes
const MAX_GIT_REQUEST_SIZE: usize = 1_000_000; // 1MB for requests (e.g., repository path, refspec)
const MAX_GIT_RESPONSE_SIZE: usize = 500_000_000; // 500MB for responses (e.g., packfiles, ls-remote output)

/// The codec for the Git exchange protocol.
#[derive(Default, Clone)]
pub struct Codec;

/// Represents possible Git requests that can be sent between peers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GitRequest {
    /// Request to clone a repository. Contains the repository URL or path.
    Clone(String),
    /// Request to fetch updates from a remote. Contains the remote name and possibly refspecs.
    Fetch(String, Option<Vec<String>>),
    /// Request to push changes to a remote. Contains the remote name and refspecs.
    Push(String, Vec<String>),
    /// Request to list remote references (e.g., `git ls-remote`).
    LsRemote(String),
    /// Request to get repository status (e.g., `git status`).
    Status(String),
}

/// Represents possible Git responses that can be sent between peers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GitResponse {
    /// Success response, often containing a confirmation message or data.
    Success(String),
    /// Failure response, with an error message.
    Error(String),
    /// Response for `LsRemote`, containing a list of remote references.
    LsRemote(Vec<(String, String)>), // (ref, oid)
    /// Response for `Status`, containing the status string.
    Status(String),
    /// Bytes data, useful for packfiles during fetch/push.
    Data(Vec<u8>),
}

impl GitResponse {
    /// Helper to check if the response is an error.
    pub fn is_error(&self) -> bool {
        matches!(self, GitResponse::Error(_))
    }
}

#[async_trait]
impl request_response::Codec for Codec {
    type Protocol = StreamProtocol;
    type Request = GitRequest;
    type Response = GitResponse;

    async fn read_request<T>(&mut self, _: &StreamProtocol, io: &mut T) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let encoded_request = read_length_prefixed(io, MAX_GIT_REQUEST_SIZE).await?;
        serde_json::from_slice(&encoded_request)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to deserialize GitRequest: {}", e)))
    }

    async fn read_response<T>(&mut self, _: &StreamProtocol, io: &mut T) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let encoded_response = read_length_prefixed(io, MAX_GIT_RESPONSE_SIZE).await?;
        serde_json::from_slice(&encoded_response)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to deserialize GitResponse: {}", e)))
    }

    async fn write_request<T>(&mut self, _: &StreamProtocol, io: &mut T, request: Self::Request) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let encoded_request = serde_json::to_vec(&request)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to serialize GitRequest: {}", e)))?;
        write_length_prefixed(io, encoded_request).await?;
        Ok(())
    }

    async fn write_response<T>(&mut self, _: &StreamProtocol, io: &mut T, response: Self::Response) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let encoded_response = serde_json::to_vec(&response)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to serialize GitResponse: {}", e)))?;
        write_length_prefixed(io, encoded_response).await?;
        Ok(())
    }
}

// --- BEGIN Utility functions (copied and adapted from file_exchange.rs) ---

/// Writes a message to the given socket with a length prefix appended to it. Also flushes the socket.
pub async fn write_length_prefixed<T>(socket: &mut T, data: impl AsRef<[u8]>) -> Result<(), io::Error>
where
    T: AsyncWrite + Unpin + Send,
{
    write_varint(socket, data.as_ref().len()).await?;
    socket.write_all(data.as_ref()).await?;
    socket.flush().await?;
    Ok(())
}

/// Writes a variable-length integer to the `socket`.
pub async fn write_varint<T>(socket: &mut T, len: usize) -> Result<(), io::Error>
where
    T: AsyncWrite + Unpin + Send,
{
    let mut len_data = unsigned_varint::encode::usize_buffer();
    let encoded_len = unsigned_varint::encode::usize(len, &mut len_data).len();
    socket.write_all(&len_data[..encoded_len]).await?;
    Ok(())
}

/// Reads a variable-length integer from the `socket`.
async fn read_varint<T>(socket: &mut T) -> Result<usize, io::Error>
where
    T: AsyncRead + Unpin + Send,
{
    let mut buffer = unsigned_varint::encode::usize_buffer();
    let mut buffer_len = 0;

    loop {
        match socket.read(&mut buffer[buffer_len..buffer_len + 1]).await? {
            0 => {
                if buffer_len == 0 {
                    return Ok(0);
                } else {
                    return Err(io::ErrorKind::UnexpectedEof.into());
                }
            }
            n => debug_assert_eq!(n, 1),
        }

        buffer_len += 1;

        match unsigned_varint::decode::usize(&buffer[..buffer_len]) {
            Ok((len, _)) => return Ok(len),
            Err(unsigned_varint::decode::Error::Overflow) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "overflow in variable-length integer",
                ));
            }
            Err(_) => {}
        }
    }
}

/// Reads a length-prefixed message from the given socket.
async fn read_length_prefixed<T>(
    socket: &mut T,
    max_size: usize,
) -> io::Result<Vec<u8>>
where
    T: AsyncRead + Unpin + Send,
{
    let len = read_varint(socket).await?;
    if len > max_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Received data size ({len} bytes) exceeds maximum ({max_size} bytes)"),
        ));
    }
    let mut buf = vec![0; len];
    socket.read_exact(&mut buf).await?;
    Ok(buf)
}

// --- END Utility functions ---
