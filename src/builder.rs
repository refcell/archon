use flate2::write::ZlibDecoder;

/// ChannelId is a 16-byte identifier for a channel.
pub type ChannelId = [u8; 16];

/// The Output Channel
#[derive(Debug)]
pub struct ChannelOut {
    /// The channel identifier.
    pub id: ChannelId,
    /// The frame number of the next frame to emit.
    /// Increment after emitting.
    pub frame: u64,
    /// The uncompressed size of the channel.
    /// Must be less than MAX_RLP_BYTES_PER_CHANNEL.
    pub rlp_length: usize,
    /// The compressor stage.
    /// Write input data to it.
    pub compress: ZlibDecoder<Vec<u8>>,
    /// The post-compression buffer.
    pub buf: Vec<u8>,
    /// Whether the channel is closed.
    pub closed: bool,
}
