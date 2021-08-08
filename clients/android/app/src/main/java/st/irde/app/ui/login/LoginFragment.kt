package st.irde.app.ui.login

import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.util.Log
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.ArrayAdapter
import android.widget.Button
import android.widget.Spinner
import android.widget.Toast
import androidx.fragment.app.Fragment
import androidx.fragment.app.FragmentTransaction
import com.google.android.material.textfield.TextInputLayout
import st.irde.app.MainActivity
import st.irde.app.R
import st.irde.app.ffi.models.UserProfile
import st.irde.app.ui.UserCreateFragment
import st.irde.app.util.AppState

/**
 * A simple [Fragment] subclass. Use the [LoginFragment.newInstance] factory method to create an
 * instance of this fragment.
 */
class LoginFragment : Fragment() {
  private val LOG_TAG = "LOGIN "
  var tcpConnected = false
  private lateinit var ctx: Context
  private lateinit var spinner: Spinner
  val localUsers: MutableList<UserProfile> = mutableListOf()

  override fun onCreateView(
    inflater: LayoutInflater,
    container: ViewGroup?,
    savedInstanceState: Bundle?
  ): View? {
    ctx = requireContext()
    return inflater.inflate(R.layout.fragment_login, container, false)
  }

  override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
    val register = view.findViewById<Button>(R.id.button_register)
    register.setOnClickListener {
      Log.d(LOG_TAG, "Pressing the register button!")
      val man = fragmentManager
      val trans = man?.beginTransaction()
      trans?.apply {
        replace(R.id.root_fragment, UserCreateFragment(this@LoginFragment)).addToBackStack(null)
        setTransition(FragmentTransaction.TRANSIT_FRAGMENT_OPEN)
        commit()
      }
    }

    // Update users once
    spinner = view.findViewById(R.id.user_list_picker)
    updateUsers()

    // TODO: 09/08/21 Awaiting server config implementation
    // Connect the TCP stack to the selected peering server
    //        val peerEntry = view.findViewById<EditText>(R.id.app_peering_server)
    //        val peerConnect = view.findViewById<Button>(R.id.peering_connect)
    //        peerConnect.setOnClickListener {
    //            val server = peerEntry.text;
    //            // TODO: add tcp-connect handshake here
    //            tcpConnected = true;
    //            peerConnect.text = getString(R.string.peering_button_disconnect)
    //            Toast.makeText(requireContext(), "Connected to server...",
    // Toast.LENGTH_LONG).show()
    //        }

    val pwEntry = view.findViewById<TextInputLayout>(R.id.password_text_input_layout)
    val login = view.findViewById<Button>(R.id.button_login)
    val TAG = "Login Tag"
    login.setOnClickListener {
      if (spinner.selectedItem != null) {
        val selected = spinner.selectedItem as UserProfile
        if (AppState.get().usersLogin(selected.id, pwEntry.editText?.text?.trim().toString())) {
          Log.d(TAG, "onViewCreated: ${pwEntry.editText?.text?.trim().toString()}")
          Toast.makeText(requireContext(), "Successfully logged in!", Toast.LENGTH_SHORT).show()
          startActivity(Intent(requireContext(), MainActivity::class.java))
        } else {
          Log.d(TAG, "onViewCreated: ${pwEntry.editText?.text.toString()}")
          Toast.makeText(requireContext(), "Wrong password!!", Toast.LENGTH_LONG).show()
        }
      }

      Log.d(LOG_TAG, "Nothing selected, can't log-in!")
    }
    super.onViewCreated(view, savedInstanceState)
  }

  private fun makeAdapter(): ArrayAdapter<UserProfile> {
    val aa = ArrayAdapter(ctx, android.R.layout.simple_spinner_item, localUsers)
    aa.setDropDownViewResource(android.R.layout.simple_spinner_dropdown_item)
    return aa
  }

  fun updateUsers() {
    localUsers.clear()
    val users = AppState.get().usersList(true)
    for (u in users) {
      localUsers.add(u)
    }
    spinner.adapter = makeAdapter()
  }
}
