package st.irde.app.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.Button
import android.widget.EditText
import android.widget.Toast
import androidx.fragment.app.Fragment
import com.google.android.material.textfield.TextInputLayout
import st.irde.app.LoginActivity
import st.irde.app.R
import st.irde.app.util.AppState

class UserCreateFragment(val login: LoginActivity) : Fragment() {
    override fun onCreateView(inflater: LayoutInflater, container: ViewGroup?, bundle: Bundle?): View? {
        val root = inflater.inflate(R.layout.fragment_register, container, false)

        val password = root.findViewById<TextInputLayout>(R.id.registry_password_entry)
        val handle = root.findViewById<TextInputLayout>(R.id.registry_handle)
        val name = root.findViewById<TextInputLayout>(R.id.registry_name)

        val button = root.findViewById<Button>(R.id.registry_create)
        button.setOnClickListener {
            val id = AppState.get().usersCreate(handle.editText?.text.toString(), name.editText?.text.toString(), password.editText?.text.toString())
            Toast.makeText(context, "Your user ID is: '${id.inner}'", Toast.LENGTH_LONG).show()

            login.updateUsers()
            fragmentManager?.popBackStack() // Go back to login!
        }

        return root
    }
}
