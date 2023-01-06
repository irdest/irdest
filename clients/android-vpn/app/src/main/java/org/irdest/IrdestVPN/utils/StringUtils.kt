package org.irdest.irdestVPN.utils

// Use Tag in any class.
val Any.TAG: String
    get() {
        return javaClass.simpleName
    }
