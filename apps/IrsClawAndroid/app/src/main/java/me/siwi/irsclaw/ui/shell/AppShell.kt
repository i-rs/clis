package me.siwi.irsclaw.ui.shell

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Book
import androidx.compose.material.icons.filled.Build
import androidx.compose.material.icons.filled.ChevronRight
import androidx.compose.material.icons.filled.Extension
import androidx.compose.material.icons.filled.Group
import androidx.compose.material.icons.filled.InsertChart
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.filled.Star
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
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import kotlinx.coroutines.launch
import me.siwi.irsclaw.logic.ConnectionState
import me.siwi.irsclaw.logic.ClawViewModel
import me.siwi.irsclaw.ui.chat.ChatScreen
import me.siwi.irsclaw.ui.chat.SessionList
import me.siwi.irsclaw.ui.components.AgentChip
import me.siwi.irsclaw.ui.panels.AgentsPanel
import me.siwi.irsclaw.ui.panels.PluginsPanel
import me.siwi.irsclaw.ui.panels.SidebarTab
import me.siwi.irsclaw.ui.panels.SkillsPanel
import me.siwi.irsclaw.ui.panels.ToolsPanel
import me.siwi.irsclaw.ui.panels.UsagePanel
import me.siwi.irsclaw.ui.settings.SettingsScreen
import me.siwi.irsclaw.ui.theme.IosColors

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
    // Settings overlays the shell regardless of connection state; back closes it.
    BackHandler(enabled = settingsVisible) {
        settingsVisible = false
    }

    Column(Modifier.fillMaxSize().safeDrawingPadding()) {
        if (settingsVisible) {
            // Settings must be reachable even while disconnected (backend URL/token setup).
            SettingsScreen(viewModel = viewModel, onBack = { settingsVisible = false })
        } else if (state.connectionState == ConnectionState.CONNECTED) {
            if (expanded) {
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
                backendName = settings?.currentBackend?.name ?: "backend",
                onConnect = { viewModel.connectToBackend() },
                onOpenSettings = { settingsVisible = true },
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
                SidebarSectionLabel("Agent")
                if (state.agents.size <= 1) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(horizontal = 12.dp, vertical = 6.dp)
                            .background(MaterialTheme.colorScheme.primary.copy(alpha = 0.08f), RoundedCornerShape(8.dp))
                            .padding(horizontal = 8.dp, vertical = 6.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        Box(
                            modifier = Modifier
                                .size(28.dp)
                                .shadow(3.dp, RoundedCornerShape(8.dp))
                                .background(Brush.linearGradient(listOf(IosColors.Blue, IosColors.Purple)), RoundedCornerShape(8.dp)),
                            contentAlignment = Alignment.Center,
                        ) {
                            Icon(Icons.Filled.Star, contentDescription = null, tint = Color.White, modifier = Modifier.size(12.dp))
                        }
                        Text(
                            text = state.currentAgentId,
                            style = MaterialTheme.typography.titleMedium.copy(fontSize = 15.sp),
                            fontWeight = FontWeight.Medium,
                            color = scheme.onSurface,
                        )
                    }
                } else {
                    Row(
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        modifier = Modifier
                            .horizontalScroll(rememberScrollState())
                            .padding(horizontal = 12.dp, vertical = 2.dp),
                    ) {
                        state.agents.forEach { agent ->
                            AgentChip(
                                agentId = agent.id,
                                selected = agent.id == state.currentAgentId,
                                onClick = { viewModel.switchAgent(agent.id) },
                            )
                        }
                    }
                }

                SidebarSectionLabel("Sessions")
                OutlinedTextField(
                    value = searchQuery,
                    onValueChange = onSearchChange,
                    placeholder = { Text("Search sessions") },
                    leadingIcon = { Icon(Icons.Filled.Search, contentDescription = null) },
                    singleLine = true,
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 12.dp),
                )
                SessionList(
                    sessions = state.sessions,
                    currentSessionId = state.currentSession?.id,
                    searchQuery = searchQuery,
                    onSelect = { viewModel.switchToSession(it.id) },
                    onDelete = { viewModel.deleteSession(it.id) },
                )
                SidebarSectionLabel("Manage")
                SidebarTab.entries.filter { it != SidebarTab.SESSIONS }.forEach { tab ->
                    ManageRow(
                        tab = tab,
                        selected = selectedTab == tab,
                        onClick = { onSelectTab(tab) },
                    )
                }
                ManageRow(tab = null, selected = false, onClick = onOpenSettings, label = "Settings")
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
private fun SidebarSectionLabel(text: String) {
    Text(
        text = text.uppercase(),
        style = MaterialTheme.typography.labelMedium,
        fontWeight = FontWeight.SemiBold,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
    )
}

private val MANAGE_ICONS = mapOf(
    SidebarTab.TOOLS to Icons.Filled.Build,
    SidebarTab.SKILLS to Icons.Filled.Book,
    SidebarTab.PLUGINS to Icons.Filled.Extension,
    SidebarTab.USAGE to Icons.Filled.InsertChart,
    SidebarTab.AGENTS to Icons.Filled.Group,
)

private val MANAGE_COLORS = mapOf(
    SidebarTab.TOOLS to IosColors.Orange,
    SidebarTab.SKILLS to IosColors.Green,
    SidebarTab.PLUGINS to IosColors.Purple,
    SidebarTab.USAGE to IosColors.Blue,
    SidebarTab.AGENTS to IosColors.Teal,
)

/** iOS ManageTabRow: tinted icon tile + label + trailing chevron when selected. */
@Composable
private fun ManageRow(tab: SidebarTab?, selected: Boolean, onClick: () -> Unit, label: String? = null) {
    val scheme = MaterialTheme.colorScheme
    val accent = tab?.let { MANAGE_COLORS[it] } ?: scheme.primary
    val icon = tab?.let { MANAGE_ICONS[it] }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
            .padding(horizontal = 12.dp, vertical = 6.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        Box(
            modifier = Modifier
                .size(28.dp)
                .background(accent.copy(alpha = if (selected) 0.2f else 0.1f), RoundedCornerShape(6.dp)),
            contentAlignment = Alignment.Center,
        ) {
            if (icon != null) {
                Icon(icon, contentDescription = null, tint = accent, modifier = Modifier.size(13.dp))
            } else {
                Icon(Icons.Filled.Settings, contentDescription = null, tint = accent, modifier = Modifier.size(13.dp))
            }
        }
        Text(
            text = label ?: tab!!.label,
            style = MaterialTheme.typography.titleMedium.copy(fontSize = 15.sp),
            fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal,
            color = if (selected) scheme.onSurface else scheme.onSurfaceVariant,
            modifier = Modifier.weight(1f),
        )
        if (selected) {
            Icon(
                Icons.Filled.ChevronRight,
                contentDescription = null,
                tint = accent,
                modifier = Modifier.size(10.dp),
            )
        }
    }
}
