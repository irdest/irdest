package st.irde.app.ui.chat

import android.view.View
import android.view.ViewGroup
import androidx.fragment.app.FragmentManager
import androidx.recyclerview.widget.RecyclerView
import kotlinx.android.synthetic.main.item_chat_room.view.*
import st.irde.app.R
import st.irde.app.ffi.models.ChatRoom
import st.irde.app.util.inflate

class ChatListAdapter(private val rooms: MutableList<ChatRoom>, private val fragMan: FragmentManager)
    : RecyclerView.Adapter<ChatListAdapter.RoomHolder>() {

    override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): RoomHolder {
        val inflated = parent.inflate(R.layout.item_chat_room, false)
        return RoomHolder(inflated, fragMan)
    }

    override fun getItemCount() = rooms.size

    override fun onBindViewHolder(holder: RoomHolder, position: Int) {
        holder.bindRoom(rooms[position])
    }

    class RoomHolder(v: View, private val man: FragmentManager)
        : RecyclerView.ViewHolder(v), View.OnClickListener {
        private var view: View = v
        var room: ChatRoom? = null

        init {
            v.setOnClickListener(this)
        }

        fun bindRoom(room: ChatRoom) {
            this.room = room

            // Then set the UI state
            view.chatroom_list_item_name.text = room.name!!
            view.chatroom_list_item_timestamp.text = room.last_message!!
            view.chatroom_list_item_unread_count.text = room.unread.toString()
        }

        override fun onClick(v: View?) {
            ChatRoomFragment(room!!).transitionInto(man)
        }
    }
}
