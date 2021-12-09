## This schema file contains types contained within a chunk but that
#    are not record-specific -- basically just the header (;

@0xdeb8161adec23ea5;

# This type encodes the chunk metadata header
#
# This is an un-encrypted type and can thus contain sensitive data
struct Header {
    maxLen @0 :UInt64;
    usage @1 :UInt64;
}
