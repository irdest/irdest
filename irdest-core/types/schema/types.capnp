@0xaa0453e951e365a5;


#############################
## General data structures
## 


struct Map(Key, Value) {
        entries @0 :List(Entry);
        struct Entry {
                key @0 :Key;
                value @1 :Value;
        }
}

struct ItemDiff(Val) {
        union {
                set @0 :Val;
                unset @1 :Void;
        }
}

struct SetDiff(Val) {
        union {
                add @0 :Val;
                remove @1 :Val;
                ignore @2 :Void;
        }
}

struct Tag {
    key @0 :Text;
    val @1 :Data;
}

struct TagSet {
    tags @0 :List(Tag);
}


#############################
## User structures
## 

struct UserAuth {
        id @0 :Text;
        token @1 :Text;
}

struct UserProfile {
        id @0 :Text;
        handle @1 :Text;
        displayName @2 :Text;
        bio @3 :Map(Text, Text);
        services @4 :List(Text);
        avatar @5 :Data;
}

struct UserUpdate {
        handle @0 :ItemDiff(Text);
        displayName @1 :ItemDiff(Text);
        addToBio @2 :List(BioLine);
        rmFromBio @3 :List(Text);
        services @4 :List(SetDiff(Text));
        aviData @5 :ItemDiff(Data);

        struct BioLine {
                key @0 :Text;
                val @1 :Text;
        }
}


#############################
## Message structures
##

struct SigTrust {
        union {
                trusted @0 :Void;
                unverified @1 :Void;
                invalid @2 :Void;
        }
}

struct Mode {
        union {
                flood @0 :Void;
                std @1 :Text;
        }
}

struct Id {
        union {
                unique @0 :Void;
                grouped @1 :Text;
        }
}

struct MsgQuery {
        id @0 :Text;
        sender @1 :Text;
        tags @2 :TagSet;
        skip @3 :UInt32;
}

struct Message {
        id @0 :Text;
        sender @1 :Text;
        associator @2 :Text;
        tags @3 :TagSet;
        payload @4 :Data;
}


#############################
## Contacts structures
##

struct ContactsEntry {
        nick @0 :Text;
        trust @1 :Int8;
        met @2 :Bool;
        location @3 :Text;
        notes @4 :Text;
}


struct ContactQuery {
    union {
        nick @0 :Text;
        trust :group {
            val @1 :Int8;
            fuz @2 :Int8;
        }
        met @3 :Bool;
        location @4 :Text;
        notes @5 :Text;
    }
}
