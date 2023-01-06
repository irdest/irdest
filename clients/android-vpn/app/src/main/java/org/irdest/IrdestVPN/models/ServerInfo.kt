package org.irdest.irdestVPN.models

import org.irdest.irdestVPN.utils.isValidIp
import org.irdest.irdestVPN.utils.isValidPort
import java.util.*

data class ServerInfo(
    var serverAddress: String,
    var serverPort: String,
    var sharedSecret: String,
)

@Throws
fun ServerInfo.isValid(): Boolean {
    if (!isValidIp(serverAddress)) {
        throw InputMismatchException("Invalid IP address [$serverAddress].")
    } else if (!isValidPort(serverPort)) {
        throw InputMismatchException("Invalid port [$serverPort].")
    } else {
        return true
    }
}
