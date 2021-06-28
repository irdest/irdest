package st.irde.app

import android.R.layout.simple_spinner_dropdown_item
import android.R.layout.simple_spinner_item
import android.content.Intent
import android.os.Bundle
import android.util.Log
import android.widget.Button
import android.widget.Spinner
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity
import androidx.fragment.app.FragmentTransaction
import com.google.android.material.textfield.TextInputLayout
import st.irde.app.ffi.models.UserProfile
import st.irde.app.net.WifiP2PService
import st.irde.app.ui.UserCreateFragment
import st.irde.app.util.AppState
import android.widget.ArrayAdapter as ArrayAdapter


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

    val LOG_TAG = "login"

    var tcpConnected = false
    val localUsers: MutableList<UserProfile> = mutableListOf()
    lateinit var spinner: Spinner

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.fragment_login)

        // TODO: Request permissions

        // Start the wifi service
        startService(Intent(this, WifiP2PService(this)::class.java))

        // Handle the register screen
        val register = findViewById<Button>(R.id.button_register)
        register.setOnClickListener {
            Log.d(LOG_TAG, "Pressing the register button!")
            val man = supportFragmentManager
            val trans = man.beginTransaction()
            trans.replace(R.id.login_container_layout, UserCreateFragment(this)).addToBackStack(null)
            trans.setTransition(FragmentTransaction.TRANSIT_FRAGMENT_OPEN)
            trans.commit()
        }

        // Update users once
        spinner = findViewById(R.id.user_list_picker)
        updateUsers()

        // Connect the TCP stack to the selected peering server
//        val peerEntry = findViewById<EditText>(R.id.app_peering_server)
//        val peerConnect = findViewById<Button>(R.id.peering_connect)
//        peerConnect.setOnClickListener {
//            val server = peerEntry.text;
//            // TODO: add tcp-connect handshake here
//            tcpConnected = true;
//            peerConnect.text = getString(R.string.peering_button_disconnect)
//            Toast.makeText(baseContext, "Connected to server...", Toast.LENGTH_LONG).show()
//        }

        val pwEntry = findViewById<TextInputLayout>(R.id.password_text_input_layout)
        val login = findViewById<Button>(R.id.button_login)
        login.setOnClickListener {
            if (spinner.selectedItem != null) {
                val selected = spinner.selectedItem as UserProfile
                if (AppState.get().usersLogin(selected.id, pwEntry.editText?.text.toString())) {
                    Toast.makeText(baseContext, "Successfully logged in!", Toast.LENGTH_SHORT).show()
                    startActivity(Intent(this, MainActivity::class.java))
                } else {
                    Toast.makeText(baseContext, "Wrong password!!", Toast.LENGTH_LONG).show()
                }
            }

            Log.d(LOG_TAG, "Nothing selected, can't log-in!")
        }
    }

    fun makeAdapter(): ArrayAdapter<UserProfile> {
        val aa = ArrayAdapter(this, simple_spinner_item, localUsers)
        aa.setDropDownViewResource(simple_spinner_dropdown_item)
        return aa
    }

    fun updateUsers() {
        localUsers.clear()
        val users = AppState.get().usersList(true)
        for (u in users) { localUsers.add(u) }
        spinner.adapter = makeAdapter()
    }
}
