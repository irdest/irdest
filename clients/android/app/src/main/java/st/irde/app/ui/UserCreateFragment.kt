package st.irde.app.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.view.WindowManager
import android.widget.Button
import android.widget.Toast
import androidx.fragment.app.Fragment
import com.google.android.material.textfield.TextInputLayout
import st.irde.app.R
import st.irde.app.ui.login.LoginFragment
import st.irde.app.util.AppState

class UserCreateFragment(private val login: LoginFragment) : Fragment() {

    private lateinit var button: Button
    private lateinit var password: TextInputLayout
    private lateinit var handle: TextInputLayout
    private lateinit var name: TextInputLayout

    override fun onCreateView(
      inflater: LayoutInflater,
      container: ViewGroup?,
      bundle: Bundle?
    ): View? {
        val root = inflater.inflate(R.layout.fragment_register, container, false)

        with(root) {
            password = findViewById(R.id.registry_password_entry)
            handle = findViewById(R.id.registry_handle)
            name = findViewById(R.id.registry_name)
            button = findViewById(R.id.registry_create)
        }
        activity?.window?.setSoftInputMode(WindowManager.LayoutParams.SOFT_INPUT_ADJUST_RESIZE)
        return root
    }

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        button.setOnClickListener {
            val id = AppState.get()
              .usersCreate(handle.editText?.text?.toString(), name.editText?.text?.trim().toString(), password.editText?.text?.trim().toString())
            Toast.makeText(context, "Your user ID is: '${id.inner}'", Toast.LENGTH_LONG).show()

            login.updateUsers()
            fragmentManager?.popBackStack() // Go back to login!
        }
        super.onViewCreated(view, savedInstanceState)
    }
}
