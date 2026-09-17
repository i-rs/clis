package me.siwi.irsclaw.ui.settings

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import me.siwi.irsclaw.data.model.ClawAgent
import me.siwi.irsclaw.data.settings.BackendConfig
import me.siwi.irsclaw.data.settings.Appearance
import me.siwi.irsclaw.logic.ClawViewModel

/**
 * Settings: 外观 / 连接 / 后端配置 / AI Provider / 智能体 / 关于 — the port of
 * the iOS SettingsView.
 */
@OptIn(ExperimentalMaterial3Api::class, androidx.compose.foundation.layout.ExperimentalLayoutApi::class)
@Composable
fun SettingsScreen(
    viewModel: ClawViewModel,
    onBack: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val settings by viewModel.settings.collectAsStateWithLifecycle()
    val scheme = MaterialTheme.colorScheme

    var editingBackend by remember { mutableStateOf<BackendConfig?>(null) }
    var showBackendEditor by remember { mutableStateOf(false) }
    var editingAgent by remember { mutableStateOf<ClawAgent?>(null) }
    var showAgentEditor by remember { mutableStateOf(false) }
    var deletingAgent by remember { mutableStateOf<ClawAgent?>(null) }

    Column(modifier = modifier.fillMaxSize()) {
        TopAppBar(
            title = { Text("设置") },
            navigationIcon = {
                IconButton(onClick = onBack) {
                    Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "返回")
                }
            },
        )
        LazyColumn(Modifier.fillMaxSize()) {
            // 外观
            item { SectionHeader("外观") }
            item {
                Row(
                    modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    Appearance.entries.forEach { appearance ->
                        FilterChip(
                            selected = settings?.appearance == appearance,
                            onClick = { viewModel.setAppearance(appearance) },
                            label = { Text(appearance.label) },
                        )
                    }
                }
            }

            // 连接
            item { SectionHeader("连接") }
            item {
                val connection = state.connectionState.name.lowercase()
                SettingsRow(
                    title = settings?.currentBackend?.name ?: "-",
                    subtitle = "${settings?.currentBackend?.url ?: "-"} · $connection",
                )
            }

            // 后端
            item { SectionHeader("后端") }
            settings?.backends?.forEach { backend ->
                item(key = "backend_${backend.id}") {
                    SettingsRow(
                        title = backend.name,
                        subtitle = backend.url,
                        selected = backend.id == settings?.currentBackend?.id,
                        onClick = {
                            viewModel.switchBackend(backend.id)
                        },
                        trailing = {
                            Row {
                                IconButton(onClick = {
                                    editingBackend = backend
                                    showBackendEditor = true
                                }) {
                                    Icon(Icons.Filled.Edit, contentDescription = "编辑", tint = scheme.onSurfaceVariant, modifier = Modifier.padding(4.dp))
                                }
                                if ((settings?.backends?.size ?: 0) > 1) {
                                    IconButton(onClick = { viewModel.deleteBackend(backend.id) }) {
                                        Icon(Icons.Filled.Delete, contentDescription = "删除", tint = scheme.onSurfaceVariant, modifier = Modifier.padding(4.dp))
                                    }
                                }
                            }
                        },
                    )
                }
            }
            item {
                TextButton(
                    onClick = {
                        editingBackend = null
                        showBackendEditor = true
                    },
                    modifier = Modifier.padding(horizontal = 16.dp),
                ) {
                    Icon(Icons.Filled.Add, contentDescription = null, modifier = Modifier.padding(end = 4.dp))
                    Text("添加后端")
                }
            }

            // AI Provider
            item { SectionHeader("AI Provider") }
            item { ProviderSection(viewModel, settings?.llmProvider ?: "", settings?.llmApiKey ?: "", settings?.llmBaseUrl ?: "") }

            // Agents
            item { SectionHeader("智能体") }
            state.agents.forEach { agent ->
                item(key = "agent_${agent.id}") {
                    SettingsRow(
                        title = agent.id,
                        subtitle = listOfNotNull(agent.provider, agent.model, "${agent.toolCount} 工具")
                            .joinToString(" · "),
                        onClick = {
                            editingAgent = agent
                            showAgentEditor = true
                        },
                        trailing = {
                            Row {
                                if (agent.id != "default") {
                                    IconButton(onClick = { deletingAgent = agent }) {
                                        Icon(Icons.Filled.Delete, contentDescription = "删除", tint = scheme.onSurfaceVariant, modifier = Modifier.padding(4.dp))
                                    }
                                }
                            }
                        },
                    )
                }
            }
            item {
                TextButton(
                    onClick = {
                        editingAgent = null
                        showAgentEditor = true
                    },
                    modifier = Modifier.padding(horizontal = 16.dp),
                ) {
                    Icon(Icons.Filled.Add, contentDescription = null, modifier = Modifier.padding(end = 4.dp))
                    Text("新建智能体")
                }
            }

            // 关于
            item { SectionHeader("关于") }
            item {
                SettingsRow(title = "i-rs Claw Android", subtitle = "版本 1.0.0 · 对接 claw serve HTTP API")
            }
            item { androidx.compose.foundation.layout.Spacer(Modifier.padding(bottom = 32.dp)) }
        }
    }

    if (showBackendEditor) {
        BackendEditorSheet(
            existing = editingBackend,
            onSave = { config ->
                viewModel.saveBackend(config)
                showBackendEditor = false
            },
            onDismiss = { showBackendEditor = false },
        )
    }
    if (showAgentEditor) {
        AgentEditorSheet(
            existing = editingAgent,
            viewModel = viewModel,
            onDismiss = { showAgentEditor = false },
        )
    }
    deletingAgent?.let { agent ->
        AlertDialog(
            onDismissRequest = { deletingAgent = null },
            title = { Text("删除智能体") },
            text = { Text("确定删除「${agent.id}」？此操作不可撤销。") },
            confirmButton = {
                TextButton(onClick = {
                    viewModel.deleteAgent(agent.id)
                    deletingAgent = null
                }) { Text("删除", color = scheme.error) }
            },
            dismissButton = {
                TextButton(onClick = { deletingAgent = null }) { Text("取消") }
            },
        )
    }
}

@Composable
private fun SectionHeader(text: String) {
    Text(
        text = text,
        style = MaterialTheme.typography.labelLarge,
        fontWeight = FontWeight.SemiBold,
        color = MaterialTheme.colorScheme.primary,
        modifier = Modifier.padding(start = 16.dp, top = 20.dp, bottom = 6.dp),
    )
}

@Composable
private fun SettingsRow(
    title: String,
    subtitle: String? = null,
    selected: Boolean = false,
    onClick: (() -> Unit)? = null,
    trailing: (@Composable () -> Unit)? = null,
) {
    val scheme = MaterialTheme.colorScheme
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 8.dp)
            .clip(RoundedCornerShape(10.dp))
            .clickable(enabled = onClick != null) { onClick?.invoke() }
            .padding(8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(Modifier.weight(1f)) {
            Text(
                text = title,
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal,
                color = if (selected) scheme.primary else scheme.onSurface,
            )
            if (subtitle != null) {
                Text(text = subtitle, style = MaterialTheme.typography.bodySmall, color = scheme.onSurfaceVariant)
            }
        }
        trailing?.invoke()
    }
    HorizontalDivider(color = scheme.outline.copy(alpha = 0.1f), modifier = Modifier.padding(horizontal = 16.dp))
}

/** Provider picker + API key + base URL, pushed to the server via PATCH /api/config. */
@OptIn(androidx.compose.foundation.layout.ExperimentalLayoutApi::class)
@Composable
private fun ProviderSection(viewModel: ClawViewModel, savedProvider: String, savedKey: String, savedUrl: String) {
    var provider by rememberSaveable { mutableStateOf(savedProvider.ifBlank { "deepseek" }) }
    var apiKey by rememberSaveable { mutableStateOf(savedKey) }
    var baseUrl by rememberSaveable { mutableStateOf(savedUrl.ifBlank { "https://api.deepseek.com" }) }
    var saved by rememberSaveable { mutableStateOf(false) }

    Column(Modifier.padding(horizontal = 16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        androidx.compose.foundation.layout.FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            PROVIDER_OPTIONS.forEach { p ->
                FilterChip(
                    selected = provider == p,
                    onClick = {
                        provider = p
                        saved = false
                        baseUrl = defaultBaseUrlFor(p)
                    },
                    label = { Text(p) },
                )
            }
        }
        OutlinedTextField(
            value = apiKey,
            onValueChange = { apiKey = it; saved = false },
            label = { Text("API Key") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
        )
        OutlinedTextField(
            value = baseUrl,
            onValueChange = { baseUrl = it; saved = false },
            label = { Text("Base URL") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
        )
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            Button(onClick = {
                viewModel.updateLlmConfig(provider, apiKey.trim(), baseUrl.trim())
                saved = true
            }) {
                Text("保存并应用")
            }
            if (saved) {
                Text("已保存 ✓", style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.primary)
            }
        }
    }
}

private fun defaultBaseUrlFor(provider: String): String = when (provider) {
    "deepseek" -> "https://api.deepseek.com"
    "openai" -> "https://api.openai.com/v1"
    "anthropic" -> "https://api.anthropic.com"
    "zhipu" -> "https://open.bigmodel.cn/api/paas/v4"
    "kimi" -> "https://api.moonshot.cn/v1"
    "ollama" -> "http://127.0.0.1:11434"
    "minimax" -> "https://api.minimax.chat/v1"
    "aliyun" -> "https://dashscope.aliyuncs.com/compatible-mode/v1"
    else -> ""
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun BackendEditorSheet(
    existing: BackendConfig?,
    onSave: (BackendConfig) -> Unit,
    onDismiss: () -> Unit,
) {
    var name by rememberSaveable { mutableStateOf(existing?.name ?: "") }
    var url by rememberSaveable { mutableStateOf(existing?.url ?: "http://") }
    var token by rememberSaveable { mutableStateOf(existing?.authToken ?: "") }

    ModalBottomSheet(onDismissRequest = onDismiss) {
        Column(
            Modifier
                .fillMaxWidth()
                .padding(horizontal = 20.dp)
                .padding(bottom = 24.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            Text(
                text = if (existing == null) "添加后端" else "编辑后端",
                style = MaterialTheme.typography.titleLarge,
                fontWeight = FontWeight.SemiBold,
            )
            OutlinedTextField(
                value = name,
                onValueChange = { name = it },
                label = { Text("名称") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
            OutlinedTextField(
                value = url,
                onValueChange = { url = it },
                label = { Text("服务地址") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
            OutlinedTextField(
                value = token,
                onValueChange = { token = it },
                label = { Text("Auth Token（如已设置）") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
            Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                Button(
                    onClick = {
                        onSave(
                            (existing ?: BackendConfig()).copy(
                                name = name.ifBlank { "未命名" },
                                url = url.trim(),
                                authToken = token.trim(),
                            ),
                        )
                    },
                    enabled = url.trim().startsWith("http"),
                ) {
                    Text("保存")
                }
                TextButton(onClick = onDismiss) { Text("取消") }
            }
        }
    }
}
