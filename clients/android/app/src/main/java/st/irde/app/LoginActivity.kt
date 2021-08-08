package st.irde.app

import android.content.Intent
import android.os.Bundle
import android.util.Log
import android.widget.Spinner
import androidx.appcompat.app.AppCompatActivity
import st.irde.app.net.WifiP2PService
import st.irde.app.ui.login.LoginFragment


/** The main login activity */
class LoginActivity : AppCompatActivity() {
    companion object {
        init {
            // The "android-support" crate creates a dynamic library called "libirdestdroid"
            // which we can include here simply via "irdestdroid" because it's being put
            // into the library search path via ~ m a g i c ~
            System.loadLibrary("irdestdroid")
        }
    }

    private val LOG_TAG = "login"

    lateinit var spinner: Spinner

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_login)
        supportFragmentManager.beginTransaction().replace(R.id.root_fragment, LoginFragment())
          .commit()

        // TODO: Request permissions

        // Start the wifi service
        startService(Intent(this, WifiP2PService(this)::class.java))

        // Handle the register screen
    }
}
