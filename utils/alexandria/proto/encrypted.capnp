## This schema file contains all types that are written to-disk

@0x8c122ce006bb551c;

## Some piece of encrypted data with a Nonce
struct Encrypted {
    nonce @0 :Data;
    data @1 :Data;
}

## A single data chunk with its own metadata header and data section
#
# The chunk header is only used to determine when to fill an existing chunk or to start a new chunk
struct Chunk {
    header @0 :Encrypted;
    data @1 :Encrypted;
}

## An index type that points to a set of chunks that is associated with a record
struct RecordIndex {
    header @0 :Encrypted;
    chunks @1 :List(Text);
}
