package me.siwi.irsclaw.ui.settings

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.siwi.irsclaw.data.model.AgentDetail
import me.siwi.irsclaw.data.model.AgentUpsertRequest
import me.siwi.irsclaw.data.model.ClawAgent
import me.siwi.irsclaw.logic.ClawViewModel

val PROVIDER_OPTIONS = listOf("openai", "anthropic", "deepseek", "minimax", "zhipu", "kimi", "aliyun", "ollama")

/**
 * Create/edit sheet for agent profiles — shared by the Agents panel and Settings,
 * mirroring the iOS AddAgentSheet + AgentDetailSheet.
 */
@OptIn(ExperimentalMaterial3Api::class, androidx.compose.foundation.layout.ExperimentalLayoutApi::class)
@Composable
fun AgentEditorSheet(
    existing: ClawAgent?,
    viewModel: ClawViewModel,
    onDismiss: () -> Unit,
) {
    val scheme = MaterialTheme.colorScheme
    var agentId by rememberSaveable { mutableStateOf(existing?.id ?: "") }
    var provider by rememberSaveable { mutableStateOf(existing?.provider ?: "") }
    var model by rememberSaveable { mutableStateOf(existing?.model ?: "") }
    var apiKey by rememberSaveable { mutableStateOf("") }
    var baseUrl by rememberSaveable { mutableStateOf(existing?.baseUrl ?: "") }
    var systemPrompt by rememberSaveable { mutableStateOf(existing?.systemPrompt ?: "") }
    var enabledTools by rememberSaveable { mutableStateOf(existing?.enabledTools?.joinToString(", ") ?: "") }
    var detail by rememberSaveable { mutableStateOf<AgentDetail?>(null) }

    LaunchedEffect(existing?.id) {
        existing?.id?.let { id -> viewModel.fetchAgentDetail(id) { d -> detail = d } }
    }

    ModalBottomSheet(onDismissRequest = onDismiss) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .verticalScroll(rememberScrollState())
                .padding(horizontal = 20.dp)
                .padding(bottom = 24.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            Text(
                text = if (existing == null) "新建智能体" else "编辑智能体 · ${existing.id}",
                style = MaterialTheme.typography.titleLarge,
                fontWeight = FontWeight.SemiBold,
            )
            if (existing == null) {
                OutlinedTextField(
                    value = agentId,
                    onValueChange = { agentId = it },
                    label = { Text("ID") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                )
            }
            Text("Provider", style = MaterialTheme.typography.labelLarge, color = scheme.onSurfaceVariant)
            androidx.compose.foundation.layout.FlowRow(
                horizontalArrangement = Arrangement.spacedBy(6.dp),
            ) {
                PROVIDER_OPTIONS.forEach { p ->
                    FilterChip(
                        selected = provider == p,
                        onClick = { provider = if (provider == p) "" else p },
                        label = { Text(p) },
                    )
                }
            }
            OutlinedTextField(
                value = model,
                onValueChange = { model = it },
                label = { Text("模型") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
            OutlinedTextField(
                value = apiKey,
                onValueChange = { apiKey = it },
                label = { Text("API Key（留空保持不变）") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
            OutlinedTextField(
                value = baseUrl,
                onValueChange = { baseUrl = it },
                label = { Text("Base URL（可选）") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
            OutlinedTextField(
                value = systemPrompt,
                onValueChange = { systemPrompt = it },
                label = { Text("系统提示词（可选）") },
                minLines = 3,
                maxLines = 6,
                modifier = Modifier.fillMaxWidth(),
            )
            OutlinedTextField(
                value = enabledTools,
                onValueChange = { enabledTools = it },
                label = { Text("启用工具（逗号分隔，留空 = 全部）") },
                minLines = 1,
                modifier = Modifier.fillMaxWidth(),
            )

            detail?.let { d ->
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(vertical = 4.dp),
                ) {
                    if (!d.mcpServers.isNullOrEmpty()) {
                        Text("MCP: ${d.mcpServers.joinToString()}", style = MaterialTheme.typography.bodySmall, color = scheme.onSurfaceVariant)
                    }
                    if (!d.allowedDirs.isNullOrEmpty()) {
                        Text("允许目录: ${d.allowedDirs.joinToString()}", style = MaterialTheme.typography.bodySmall, color = scheme.onSurfaceVariant)
                    }
                }
            }

            Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                Button(
                    onClick = {
                        val request = AgentUpsertRequest(
                            id = if (existing == null) agentId.trim() else existing.id,
                            provider = provider.ifBlank { null },
                            model = model.ifBlank { null },
                            apiKey = apiKey.ifBlank { null },
                            baseUrl = baseUrl.ifBlank { null },
                            systemPrompt = systemPrompt.ifBlank { null },
                            enabledTools = enabledTools.split(",").map { it.trim() }.filter { it.isNotEmpty() }
                                .takeIf { it.isNotEmpty() },
                        )
                        if (existing == null) {
                            viewModel.createAgent(request) { onDismiss() }
                        } else {
                            viewModel.updateAgent(existing.id, request) { onDismiss() }
                        }
                    },
                    enabled = existing != null || agentId.isNotBlank(),
                ) {
                    Text("保存")
                }
                androidx.compose.material3.OutlinedButton(onClick = onDismiss) {
                    Text("取消")
                }
            }
        }
    }
}
