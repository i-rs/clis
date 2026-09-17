package me.siwi.irsclaw.ui.shell

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.DrawerValue
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalDrawerSheet
import androidx.compose.material3.ModalNavigationDrawer
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.rememberDrawerState
import androidx.compose.material3.windowsizeclass.WindowWidthSizeClass
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import kotlinx.coroutines.launch
import me.siwi.irsclaw.logic.ConnectionState
import me.siwi.irsclaw.logic.ClawViewModel
import me.siwi.irsclaw.ui.chat.ChatScreen
import me.siwi.irsclaw.ui.chat.SessionList
import me.siwi.irsclaw.ui.panels.AgentsPanel
import me.siwi.irsclaw.ui.panels.PluginsPanel
import me.siwi.irsclaw.ui.panels.SidebarTab
import me.siwi.irsclaw.ui.panels.SkillsPanel
import me.siwi.irsclaw.ui.panels.ToolsPanel
import me.siwi.irsclaw.ui.panels.UsagePanel
import me.siwi.irsclaw.ui.settings.SettingsScreen

/**
 * Adaptive app shell: phones get a modal drawer (iPhone pattern); expanded-width
 * screens get a permanent sidebar with a detail pane (iPad split pattern).
 */
@Composable
fun AppShell(viewModel: ClawViewModel, windowWidth: WindowWidthSizeClass) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val settings by viewModel.settings.collectAsStateWithLifecycle()
    val expanded = windowWidth != WindowWidthSizeClass.Compact

    var selectedTab by rememberSaveable { mutableStateOf(SidebarTab.SESSIONS) }
    var settingsVisible by rememberSaveable { mutableStateOf(false) }
    var searchQuery by rememberSaveable { mutableStateOf("") }

    val openSettings = { settingsVisible = true }

    // On phones, non-chat tabs are full-screen panels; back returns to chat.
    BackHandler(enabled = !expanded && !settingsVisible && selectedTab != SidebarTab.SESSIONS) {
        selectedTab = SidebarTab.SESSIONS
    }

    Column(Modifier.fillMaxSize().safeDrawingPadding()) {
        if (state.connectionState == ConnectionState.CONNECTED) {
            if (settingsVisible) {
                SettingsScreen(viewModel = viewModel, onBack = { settingsVisible = false })
            } else if (expanded) {
                TabletSplitBody(
                    viewModel = viewModel,
                    selectedTab = selectedTab,
                    onSelectTab = { selectedTab = it },
                    searchQuery = searchQuery,
                    onSearchChange = { searchQuery = it },
                    onOpenSettings = openSettings,
                )
            } else {
                PhoneDrawerBody(
                    viewModel = viewModel,
                    selectedTab = selectedTab,
                    onSelectTab = { selectedTab = it },
                    onOpenSettings = openSettings,
                )
            }
        } else {
            ConnectScreen(
                state = state.connectionState,
                errorMessage = state.errorMessage,
                backendName = settings?.currentBackend?.name ?: "后端",
                onConnect = { viewModel.connectToBackend() },
            )
        }
    }

    LaunchedEffect(Unit) {
        if (state.connectionState == ConnectionState.DISCONNECTED) {
            viewModel.connectToBackend()
        }
    }
}

// MARK: - Phone (modal drawer, iPhone pattern)

@Composable
private fun PhoneDrawerBody(
    viewModel: ClawViewModel,
    selectedTab: SidebarTab,
    onSelectTab: (SidebarTab) -> Unit,
    onOpenSettings: () -> Unit,
) {
    val drawerState = rememberDrawerState(DrawerValue.Closed)
    val scope = rememberCoroutineScope()

    ModalNavigationDrawer(
        drawerState = drawerState,
        drawerContent = {
            ModalDrawerSheet {
                DrawerContent(
                    viewModel = viewModel,
                    onNavigate = { tab ->
                        onSelectTab(tab)
                        scope.launch { drawerState.close() }
                    },
                    onSelectSession = { session ->
                        viewModel.switchToSession(session.id)
                        onSelectTab(SidebarTab.SESSIONS)
                        scope.launch { drawerState.close() }
                    },
                )
            }
        },
    ) {
        when (selectedTab) {
            SidebarTab.SESSIONS -> ChatScreen(
                viewModel = viewModel,
                onMenuClick = { scope.launch { drawerState.open() } },
                onOpenSettings = onOpenSettings,
            )
            else -> TabContent(viewModel, selectedTab, onBack = { onSelectTab(SidebarTab.SESSIONS) })
        }
    }
}

// MARK: - Tablet (permanent sidebar + detail, iPad split pattern)

@Composable
private fun TabletSplitBody(
    viewModel: ClawViewModel,
    selectedTab: SidebarTab,
    onSelectTab: (SidebarTab) -> Unit,
    searchQuery: String,
    onSearchChange: (String) -> Unit,
    onOpenSettings: () -> Unit,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val scheme = MaterialTheme.colorScheme

    Row(Modifier.fillMaxSize()) {
        Surface(modifier = Modifier.width(300.dp).fillMaxSize(), color = scheme.surfaceContainer) {
            Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState())) {
                Text(
                    text = "i-rs Claw",
                    style = MaterialTheme.typography.titleLarge,
                    fontWeight = FontWeight.SemiBold,
                    modifier = Modifier.padding(horizontal = 16.dp, vertical = 10.dp),
                )
                OutlinedTextField(
                    value = searchQuery,
                    onValueChange = onSearchChange,
                    placeholder = { Text("搜索会话") },
                    leadingIcon = { Icon(Icons.Filled.Search, contentDescription = null) },
                    singleLine = true,
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 12.dp),
                )
                SectionLabel("会话")
                SessionList(
                    sessions = state.sessions,
                    currentSessionId = state.currentSession?.id,
                    searchQuery = searchQuery,
                    onSelect = { viewModel.switchToSession(it.id) },
                    onDelete = { viewModel.deleteSession(it.id) },
                )
                SectionLabel("管理")
                SidebarTab.entries.filter { it != SidebarTab.SESSIONS }.forEach { tab ->
                    SidebarRow(label = tab.label, selected = selectedTab == tab) { onSelectTab(tab) }
                }
                SidebarRow(label = "设置", selected = false, onClick = onOpenSettings)
            }
        }
        Surface(Modifier.weight(1f).fillMaxSize(), color = scheme.background) {
            when (selectedTab) {
                SidebarTab.SESSIONS -> ChatScreen(
                    viewModel = viewModel,
                    onMenuClick = null,
                    onOpenSettings = onOpenSettings,
                )
                else -> TabContent(viewModel, selectedTab, onBack = null)
            }
        }
    }
}

// MARK: - Shared pieces

/** A manage tab rendered full-screen (phone) or in the detail pane (tablet). */
@OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
@Composable
private fun TabContent(viewModel: ClawViewModel, tab: SidebarTab, onBack: (() -> Unit)?) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    Column(Modifier.fillMaxSize()) {
        if (onBack != null) {
            TopAppBar(
                title = { Text(tab.label) },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "返回对话")
                    }
                },
            )
        }
        when (tab) {
            SidebarTab.TOOLS -> ToolsPanel(tools = state.tools)
            SidebarTab.SKILLS -> SkillsPanel(skills = state.skills)
            SidebarTab.PLUGINS -> PluginsPanel(plugins = state.plugins)
            SidebarTab.USAGE -> UsagePanel(viewModel = viewModel)
            SidebarTab.AGENTS -> AgentsPanel(viewModel = viewModel)
            SidebarTab.SESSIONS -> Unit
        }
    }
}

@Composable
private fun SectionLabel(text: String) {
    Text(
        text = text,
        style = MaterialTheme.typography.labelMedium,
        fontWeight = FontWeight.SemiBold,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
    )
}

@Composable
private fun SidebarRow(label: String, selected: Boolean, onClick: () -> Unit) {
    val scheme = MaterialTheme.colorScheme
    Text(
        text = label,
        style = MaterialTheme.typography.bodyMedium,
        fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal,
        color = if (selected) scheme.primary else scheme.onSurface,
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
            .padding(horizontal = 16.dp, vertical = 10.dp),
    )
}
