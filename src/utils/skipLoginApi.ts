/**
 * 跳过登录功能 API
 * 
 * 提供 Claude Code、Codex、Gemini CLI 首次安装时跳过登录的功能。
 * 用户只需配置 API Key，即可直接使用，无需进行 OAuth 登录。
 */

import { invoke } from '@tauri-apps/api/core'

// ============================================================================
// Claude Code 跳过登录
// ============================================================================

/**
 * 设置 Claude Code 跳过首次登录确认
 * 写入 ~/.claude.json 中的 hasCompletedOnboarding: true
 * 
 * @returns 是否成功修改（如果已经是 true，返回 false）
 */
export async function setClaudeCodeSkipOnboarding(): Promise<boolean> {
  return await invoke<boolean>('set_claude_code_skip_onboarding')
}

/**
 * 清除 Claude Code 跳过首次登录确认
 * 删除 ~/.claude.json 中的 hasCompletedOnboarding 字段
 * 
 * @returns 是否成功删除（如果字段不存在，返回 false）
 */
export async function clearClaudeCodeSkipOnboarding(): Promise<boolean> {
  return await invoke<boolean>('clear_claude_code_skip_onboarding')
}

/**
 * 获取 Claude Code 跳过首次登录确认状态
 * 
 * @returns hasCompletedOnboarding 的当前值
 */
export async function getClaudeCodeSkipOnboarding(): Promise<boolean> {
  return await invoke<boolean>('get_claude_code_skip_onboarding')
}

// ============================================================================
// Codex 跳过登录
// ============================================================================

/**
 * 设置 Codex API Key 并跳过 OAuth 登录
 * 直接配置 API Key 和 model_provider，避免 OAuth 登录流程
 * 
 * @param apiKey API Key
 * @param baseUrl API Base URL
 * @param providerName 服务商名称
 */
export async function setCodexApiKeySkipOAuth(
  apiKey: string,
  baseUrl: string,
  providerName: string
): Promise<void> {
  await invoke('set_codex_api_key_skip_oauth', {
    apiKey,
    baseUrl,
    providerName
  })
}

/**
 * 清除 Codex API Key 配置（恢复 OAuth 登录）
 */
export async function clearCodexApiKey(): Promise<void> {
  await invoke('clear_codex_api_key')
}

/**
 * 获取 Codex 当前 API Key
 * 
 * @returns API Key 或 null
 */
export async function getCodexApiKey(): Promise<string | null> {
  return await invoke<string | null>('get_codex_api_key')
}

// ============================================================================
// Gemini 跳过登录
// ============================================================================

/**
 * 设置 Gemini 认证模式为 API Key（跳过 OAuth 登录）
 * 写入 settings.json 中的 security.auth.selectedType: "gemini-api-key"
 */
export async function setGeminiApiKeyAuthMode(): Promise<void> {
  await invoke('set_gemini_api_key_auth_mode')
}

/**
 * 设置 Gemini 认证模式为 OAuth（Google 官方）
 * 写入 settings.json 中的 security.auth.selectedType: "oauth-personal"
 */
export async function setGeminiOAuthAuthMode(): Promise<void> {
  await invoke('set_gemini_oauth_auth_mode')
}

/**
 * 获取 Gemini 当前认证类型
 * 
 * @returns "gemini-api-key" | "oauth-personal" | null
 */
export async function getGeminiAuthSelectedType(): Promise<string | null> {
  return await invoke<string | null>('get_gemini_auth_selected_type')
}

/**
 * 清除 Gemini 认证类型设置
 */
export async function clearGeminiAuthSelectedType(): Promise<void> {
  await invoke('clear_gemini_auth_selected_type')
}

// ============================================================================
// 便捷方法：一键配置跳过登录
// ============================================================================

/**
 * 为 Claude Code 配置 API Key 并跳过登录
 * 
 * @param apiKey API Key
 * @param baseUrl API Base URL（可选，默认使用官方）
 */
export async function setupClaudeCodeWithApiKey(
  apiKey: string,
  baseUrl?: string
): Promise<void> {
  // 1. 设置 API Key
  await invoke('set_claude_code_api_key', { apiKey })
  
  // 2. 设置 Base URL（如果提供）
  if (baseUrl) {
    await invoke('set_claude_code_base_url', { baseUrl })
  }
  
  // 3. 设置跳过首次登录确认
  await setClaudeCodeSkipOnboarding()
}

/**
 * 为 Codex 配置 API Key 并跳过登录
 * 
 * @param apiKey API Key
 * @param baseUrl API Base URL
 * @param providerName 服务商名称
 */
export async function setupCodexWithApiKey(
  apiKey: string,
  baseUrl: string,
  providerName: string = 'Custom'
): Promise<void> {
  await setCodexApiKeySkipOAuth(apiKey, baseUrl, providerName)
}

/**
 * 为 Gemini 配置 API Key 并跳过登录
 * 
 * @param apiKey API Key
 * @param baseUrl API Base URL（可选）
 */
export async function setupGeminiWithApiKey(
  apiKey: string,
  baseUrl?: string
): Promise<void> {
  // 1. 设置 API Key
  await invoke('set_gemini_api_key', { apiKey })
  
  // 2. 设置 Base URL（如果提供）
  if (baseUrl) {
    await invoke('set_gemini_base_url', { baseUrl })
  }
  
  // 3. 设置认证模式为 API Key
  await setGeminiApiKeyAuthMode()
}
