package org.irdest.irdestVPN.models

enum class ConnectionState(val code: Int) {
    CONNECTING(0),
    CONNECTED(1),
    DISCONNECTED(2),
    ERROR(3),
    IDLE(4),
}

fun getConnectionStateFromCode(code: Int): ConnectionState {
    return when (code) {
        0 -> ConnectionState.CONNECTING
        1 -> ConnectionState.CONNECTED
        2 -> ConnectionState.DISCONNECTED
        3 -> ConnectionState.ERROR
        else -> ConnectionState.IDLE
    }
}

data class VpnState(
    val connectionState: ConnectionState,
    val stateMsg: String,
){
    constructor() : this(ConnectionState.IDLE, "")
}

