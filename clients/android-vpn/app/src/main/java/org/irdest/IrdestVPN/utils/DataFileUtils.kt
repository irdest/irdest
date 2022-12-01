package org.irdest.IrdestVPN.utils

import android.content.Context
import android.util.Log
import java.io.File
import java.nio.file.Path
import java.util.*

private const val TAG = "DataFileHelper"

enum class FileName(val fileName: String) {
    RATMAND_DATA("users.json")
}

fun createRatmandDataFileIfNotExist (context: Context) {
    val ratmandDataFileName = FileName.RATMAND_DATA.fileName
    createFileIfNotExist(context.filesDir, ratmandDataFileName)
}

fun createFileIfNotExist(parent: File, child: String) {
    val file = File(parent, child)
    if (!file.exists()) {
        file.createNewFile()
    }
}
