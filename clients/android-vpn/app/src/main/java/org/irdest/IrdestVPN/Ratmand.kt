package org.irdest.IrdestVPN

class Ratmand {
    // JNI constructs the name of function that it will call is
    // Java_<domain>_<class>_<methodname> (Java_org_irdest_ratman_Ratmand_ratrun).
    private external fun ratrun(test_str: String): String
}