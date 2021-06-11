package st.irde.app.util

import st.irde.app.ffi.NativeIrdest
import st.irde.app.ffi.models.Id
import st.irde.app.ffi.models.UserProfile

object AppState {
    lateinit var self: UserProfile
    private var irdestCore: NativeIrdest? = null

    private val usersCache: MutableMap<Id, UserProfile> = mutableMapOf<Id, UserProfile>()

    /**
     * Get the current native state and initialise it first
     */
    fun get(): NativeIrdest = if (irdestCore != null) {
        irdestCore!!
    } else {
        irdestCore = NativeIrdest(0)
        irdestCore!!
    }

    /**
     * Resolve a user ID to the user profile.  Fetch from Rust code if not in cache
     */
    fun getUserProfile(id: Id): UserProfile? {
        return this.usersCache[id]
    }
}
