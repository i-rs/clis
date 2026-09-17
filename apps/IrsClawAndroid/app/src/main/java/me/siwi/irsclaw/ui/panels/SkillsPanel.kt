package me.siwi.irsclaw.ui.panels

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.spring
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Book
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.KeyboardArrowRight
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.foundation.text.selection.SelectionContainer
import me.siwi.irsclaw.data.model.SkillInfo
import me.siwi.irsclaw.ui.components.CardChrome
import me.siwi.irsclaw.ui.theme.IosColors
import me.siwi.irsclaw.ui.theme.MonoStyle

/** Collapsible skill cards (iOS SkillsPanel): blue→purple tile + monospaced content. */
@Composable
fun SkillsPanel(skills: List<SkillInfo>, modifier: Modifier = Modifier) {
    if (skills.isEmpty()) {
        SectionEmpty("没有可用技能", modifier)
        return
    }
    val dark = isSystemInDarkTheme()
    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = androidx.compose.foundation.layout.PaddingValues(20.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        items(skills, key = { it.name }) { skill ->
            SkillCard(skill, dark)
        }
    }
}

@Composable
private fun SkillCard(skill: SkillInfo, dark: Boolean) {
    var expanded by rememberSaveable { mutableStateOf(false) }
    val scheme = MaterialTheme.colorScheme
    Column(
        modifier = Modifier
            .animateContentSize(spring(dampingRatio = 0.85f, stiffness = 380f))
            .shadow(8.dp, RoundedCornerShape(14.dp), spotColor = CardChrome.shadowColor())
            .background(CardChrome.background(), RoundedCornerShape(14.dp))
            .border(
                1.dp,
                CardChrome.border(IosColors.Blue, dark),
                RoundedCornerShape(14.dp),
            ),
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .clickable { expanded = !expanded }
                .padding(14.dp),
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(
                modifier = Modifier
                    .size(36.dp)
                    .shadow(4.dp, RoundedCornerShape(10.dp), spotColor = IosColors.Blue.copy(alpha = if (dark) 0.25f else 0.15f))
                    .background(
                        androidx.compose.ui.graphics.Brush.linearGradient(
                            listOf(
                                IosColors.Blue.copy(alpha = if (dark) 0.3f else 0.2f),
                                IosColors.Purple.copy(alpha = if (dark) 0.25f else 0.15f),
                            ),
                        ),
                        RoundedCornerShape(10.dp),
                    ),
                contentAlignment = Alignment.Center,
            ) {
                Icon(Icons.Filled.Book, contentDescription = null, tint = IosColors.Blue, modifier = Modifier.size(14.dp))
            }
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
                Text(
                    text = skill.name,
                    style = MaterialTheme.typography.titleMedium.copy(fontSize = 15.sp),
                    fontWeight = FontWeight.Medium,
                    color = scheme.onSurface,
                )
                Text(
                    text = skill.description,
                    style = MaterialTheme.typography.bodySmall,
                    color = scheme.onSurfaceVariant,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            Icon(
                if (expanded) Icons.Filled.KeyboardArrowDown else Icons.Filled.KeyboardArrowRight,
                contentDescription = null,
                tint = scheme.onSurfaceVariant.copy(alpha = 0.7f),
                modifier = Modifier.size(12.dp),
            )
        }
        AnimatedVisibility(visible = expanded) {
            Column {
                HorizontalDivider(modifier = Modifier.padding(horizontal = 14.dp))
                SelectionContainer {
                    Text(
                        text = skill.content,
                        style = MonoStyle,
                        color = scheme.onSurfaceVariant,
                        modifier = Modifier
                            .fillMaxWidth()
                            .heightIn(max = 300.dp)
                            .verticalScroll(rememberScrollState())
                            .padding(14.dp),
                    )
                }
            }
        }
    }
}
