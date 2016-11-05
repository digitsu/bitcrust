//!
//! Bitcrust block main documentation

/// 2016 Tomas, no rights reserved, no warranties given


use std::convert;

use hash;
use decode;
use decode::Parse;
use transaction::ParsedTx;

#[derive(Debug)]
pub enum BlockError {
    NoTransanctions,
    FirstNotCoinbase,
    DoubleCoinbase,

    UnexpectedEndOfBuffer,
}

impl convert::From<decode::EndOfBufferError> for BlockError {
    fn from(_: decode::EndOfBufferError) -> BlockError {
        BlockError::UnexpectedEndOfBuffer
    }

}

type BlockResult<T> = Result<T, BlockError>;


#[derive(Debug)]
pub struct Block<'a> {


    pub header: BlockHeader<'a>,

    pub txcount: usize,

    pub txs:    &'a[u8],

    // the full block as slice
    raw:        &'a[u8],
}

/// BlockHeader represents the header of a block
#[derive(Debug)]
pub struct BlockHeader<'a> {

    version:     u32,
    prev_hash:   hash::Hash32<'a>,
    merkle_root: hash::Hash32<'a>,
    time:        u32,
    bits:        u32,
    nonce:       u32,

    raw:         &'a[u8],
}

impl<'a> Block<'a> {

    /// Parses the block from a raw blob
    ///
    /// The transactions will not be parsed yet, and simply stored as a slice
    pub fn new(raw: &'a[u8]) -> Result<Block<'a>, decode::EndOfBufferError> {

        let mut buf = decode::Buffer::new(raw);

        Ok(Block {
            raw:     raw,
            header:  BlockHeader::parse(&mut buf)?,
            txcount: buf.parse_compact_size()?,
            txs:     buf.inner

        })
    }


    pub fn verify(&self) -> BlockResult<()> {



        Ok(())
    }

    /// Parses each transaction in the block, and executes the callback for each
    ///
    /// This will also check whether only the first transaction is a coinbase
    ///
    pub fn process_transactions<F>(&self, mut callback: F) -> BlockResult<()>
        where F : FnMut(ParsedTx<'a>) -> BlockResult<()> {

        if self.txs.is_empty() {
            return Err(BlockError::NoTransanctions);
        }


        let mut buffer = decode::Buffer::new(self.txs);

        // check if the first is coinbase
        let first_tx = ParsedTx::parse(&mut buffer)?;
        if !first_tx.is_coinbase() {
            return Err(BlockError::FirstNotCoinbase);
        }

        callback(first_tx)?;

        for _ in 1..self.txcount {
            let tx = ParsedTx::parse(&mut buffer)?;
            if tx.is_coinbase() {

                return Err(BlockError::DoubleCoinbase);
            }

            callback(tx)?;
        }

        if buffer.len() > 0  {

            // Buffer not fully consumed
            Err(BlockError::UnexpectedEndOfBuffer)
        }
            else {
                Ok(())
            }
    }
}


impl<'a> decode::Parse<'a> for BlockHeader<'a> {

    /// Parses the block-header
    fn parse(buffer: &mut decode::Buffer<'a>) -> Result<BlockHeader<'a>, decode::EndOfBufferError> {

        let org_buffer = *buffer;

        Ok(BlockHeader {
            version:     u32::parse(buffer)?,
            prev_hash:   try!(hash::Hash32::parse(buffer)),
            merkle_root: try!(hash::Hash32::parse(buffer)),
            time:        try!(u32::parse(buffer)),
            bits:        try!(u32::parse(buffer)),
            nonce:       try!(u32::parse(buffer)),

            raw:         buffer.consumed_since(org_buffer).inner
        })
    }
}

impl<'a> decode::ToRaw<'a> for BlockHeader<'a> {
    fn to_raw(&self) -> &[u8] {
        self.raw
    }
}



#[cfg(test)]
mod tests {

    extern crate rustc_serialize;

    use super::*;
    use std::io::Cursor;
    use self::rustc_serialize::hex::FromHex;
    use std::mem;
    use lmdb_rs::{EnvBuilder, DbFlags};
    use decode::Parse;
    use decode;
    use transaction;

    #[test]
    fn test_blockheader_read()  {
        let hex = "0100000000000000000000000000000000000000000000000000000000000000\
                   000000003ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa\
                   4b1e5e4a29ab5f49ffff001d1dac2b7c01010000000100000000000000000000\
                   00000000000000000000000000000000000000000000ffffffff4d04ffff001d\
                   0104455468652054696d65732030332f4a616e2f32303039204368616e63656c\
                   6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f75742066\
                   6f722062616e6b73ffffffff0100f2052a01000000434104678afdb0fe554827\
                   1967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4\
                   f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac00000000";


        let slice = &rustc_serialize::hex::FromHex::from_hex(hex).unwrap();
        let mut buf = decode::Buffer::new(slice);

        let hdr = BlockHeader::parse(&mut buf).unwrap();
        let txs: Vec<transaction::ParsedTx> = Vec::parse(&mut buf).unwrap();

        for tx in &txs {
            tx.verify_syntax().unwrap();
        }
        
        assert_eq!(hdr.version, 1);
        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].txs_in.len(), 1);
        assert_eq!(txs[0].txs_out.len(), 1);

    }


    /*
    #[test]
    fn test_blockheader_store()  {
        let hex = "0100000000000000000000000000000000000000000000000000000000000000\
                   000000003ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa\
                   4b1e5e4a29ab5f49ffff001d1dac2b7c01010000000100000000000000000000\
                   00000000000000000000000000000000000000000000ffffffff4d04ffff001d\
                   0104455468652054696d65732030332f4a616e2f32303039204368616e63656c\
                   6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f75742066\
                   6f722062616e6b73ffffffff0100f2052a01000000434104678afdb0fe554827\
                   1967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4\
                   f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac00000000";
                   
        let blk_bytes = hex.from_hex().unwrap();     
        let blk       = super::Block::read(&mut Cursor::new(&blk_bytes)).unwrap();
              
        let path = Path::new("test-lmdb");
        let env = EnvBuilder::new().open(&path, 0o777).unwrap();
        
        let db_handle = env.get_default_db(DbFlags::empty()).unwrap();
        let txn = env.new_transaction().unwrap();
        {
            let xx        = unsafe { mem::transmute(&blk) };
        
            let db = txn.bind(&db_handle); // get a database bound to this transaction
            db.set(&"test", &xx);
            
        }
        txn.commit().unwrap();
        let reader = env.get_reader().unwrap();
        let db = reader.bind(&db_handle);
        //assert_eq!(1,0);
            
        let name = db.get::<&str>(&"test").unwrap();
        assert_eq!(&name, &"aa");
    }
    */
}
