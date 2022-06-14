package org.irdest.ratman

import org.irdest.ratman.Ratmand

class Ratmand {
    // JNI constructs the name of function that it will call is
    // Java_<domain>_<class>_<methodname> (Java_org_irdest_ratman_Ratmand_ratrun).
    private external fun ratrun(test_str: String): String

    fun run_ratmand(test_string: String?): String {
        return ratrun("From android")
    }
}