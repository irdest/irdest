package org.irdest.IrdestVPN.utils

import android.content.Context
import android.content.res.Resources
import android.util.Log
import org.irdest.IrdestVPN.R
import java.io.File
import java.io.IOException
import java.util.*

private const val TAG = "DataFileHelper"

fun createFileIfNotExist(context: Context, name: String) {
    getPropertiesValue(context, name)?.let { s ->
        try {
            File(context.filesDir, s).createNewFile().let {
                if(it) Log.i(TAG, "createFileIfNotExist: New [ $s ] file is created.")
            }
        } catch (e: Exception) {
            throw e
        }
    }?: run {Log.e(TAG, "createFileIfNotExist: Failed to read properties value for $name")}
}

private fun getPropertiesValue(context: Context, name: String) : String? {
    val pInputStream = context.resources.openRawResource(R.raw.config)

    try {
        return Properties().run {
            load(pInputStream)
            getProperty(name)
        }
    } catch (e: Resources.NotFoundException) {
        Log.e(TAG, "getConfigFileName: Can not find the res/raw/config.properties file", e)
    } catch (e: IOException) {
        Log.e(TAG, "getConfigFileName: Failed to open config file.", e)
    }
    return null
}
