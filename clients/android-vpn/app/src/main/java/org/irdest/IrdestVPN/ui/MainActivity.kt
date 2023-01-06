package org.irdest.irdestVPN.ui

import android.os.Bundle
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import org.irdest.irdestVPN.utils.ServerInfoRepository
import org.irdest.irdestVPN.vpn.VpnServiceStarter
import androidx.activity.viewModels
import org.irdest.irdestVPN.databinding.ActivityMainBinding
import org.irdest.irdestVPN.utils.getBaseMessageOfState
import org.irdest.irdestVPN.utils.getColorOfState

class MainActivity : AppCompatActivity() {

    private val binding: ActivityMainBinding by lazy {
        ActivityMainBinding.inflate(layoutInflater)
    }
    private val viewModel: MainViewModel by viewModels {
        MainViewModel.MainViewModelFactory(
            ServerInfoRepository.getInstance(this.applicationContext),
            VpnServiceStarter.getInstance(this.applicationContext)
        )
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(binding.root)

        binding.lifecycleOwner = this
        binding.mainViewModel = viewModel

        val permissionActivityLauncher =
            registerForActivityResult(ActivityResultContracts.StartActivityForResult()) { result ->
                viewModel.handlePermissionActivityResult(result)
            }

        viewModel.observePermissionIntent().observe(this) {
            it?.let {
                permissionActivityLauncher.launch(it)
            }
        }

        viewModel.observeVpnState().observe(this) {
            binding.colorStateView.setBackgroundResource(getColorOfState(it.connectionState))
            binding.colorStateView.text = getString(getBaseMessageOfState(it.connectionState))
            binding.dashboardView.text = it.stateMsg
        }
    }

//    companion object {
//        init {
//            System.loadLibrary("ratman_android")
//        }
//    }
}