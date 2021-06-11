package st.irde.app.ui.chat

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.Button
import android.widget.EditText
import androidx.fragment.app.Fragment
import androidx.fragment.app.FragmentManager
import androidx.fragment.app.FragmentTransaction
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import st.irde.app.R
import st.irde.app.ffi.models.ChatMessage
import st.irde.app.ffi.models.ChatRoom
import st.irde.app.ffi.models.Id
import st.irde.app.util.AppState


class ChatRoomFragment(val room: ChatRoom) : Fragment() {
    private lateinit var layouter: LinearLayoutManager
    private lateinit var adapter: ChatRoomAdapter

    fun transitionInto(man: FragmentManager) {
        // Create transaction to replace main container view
        val trans = man.beginTransaction()
        trans.replace(R.id.nav_host_fragment, this).addToBackStack(null)
        trans.setTransition(FragmentTransaction.TRANSIT_FRAGMENT_OPEN)
        trans.commit()
    }

    override fun onCreateView(inflater: LayoutInflater, container: ViewGroup?, bundle: Bundle?): View? {
        val root = inflater.inflate(R.layout.fragment_chatroom, container, false)
        layouter = LinearLayoutManager(context)

        // TODO: add a back button maybe?

        // Some messages
        val messages: MutableList<ChatMessage> = mutableListOf(
            ChatMessage(Id("Spacekookie"), "15:11", "Hey, how are you?", Id("1")),
            ChatMessage(Id("Zelf"), "15:32", "Not bad, kinda stressed", Id("0")),
            ChatMessage(Id("Milan"), "15:33", "Trying to get this app to work", Id("0")),
            ChatMessage(Id("Arthur"), "15:36", "Yea? What's the problem?", Id("1")),
            ChatMessage(Id("Ayush"), "15:41", "There's just so many things that don't work properly and Android " +
                        "has the tendency to layer lots of abstractions on top of each other, and trying to get them all to play nice is really annoying." +
                        "\n\n" +
                        "Really, I wish I could just not do any of this >.>", Id("0"))
        )

        adapter = ChatRoomAdapter(messages)
        val chatMessageList = root.findViewById<RecyclerView>(R.id.chatroom_message_list)
        chatMessageList.adapter = adapter
        chatMessageList.layoutManager = LinearLayoutManager(context)

        val textBox = root.findViewById<EditText>(R.id.chatroom_message_box)
        val sendButton = root.findViewById<Button>(R.id.chatroom_message_send)
        sendButton.setOnClickListener {
            val msg = ChatMessage(Id(""), "Now", textBox.text.toString(), AppState.self.id)
            adapter.addMessage(msg)
            textBox.text.clear()
        }

        return root
    }
}
