package org.irdest.irdestVPN.vpn

data class TunConfigData(
    val Address: Pair<String, Int>,
    val Route: Pair<String, Int>,
    val MTU: Int,
    val DNS: String,
)
