#!/usr/bin/env node
/**
 * 图标生成脚本
 * 使用 sharp 将 SVG 转换为 PNG，并调用 tauri icon 生成平台图标
 *
 * 安装依赖: npm install sharp --save-dev
 * 运行: node scripts/generate-icons.js
 */

import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { spawnSync } from 'node:child_process'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)

async function generateIcons() {
    let sharp
    try {
        const mod = await import('sharp')
        sharp = mod.default ?? mod
    } catch (e) {
        console.error('请先安装 sharp: npm install sharp --save-dev')
        process.exit(1)
    }

    const iconsDir = path.join(__dirname, '..', 'src-tauri', 'icons')
    const svgPath = path.join(iconsDir, 'app-icon.svg')

    if (!fs.existsSync(svgPath)) {
        console.error('SVG 图标不存在:', svgPath)
        process.exit(1)
    }

    console.log('正在生成图标...')

    // PNG 尺寸列表
    const sizes = [32, 128, 256, 512]

    for (const size of sizes) {
        const outputPath = path.join(iconsDir, size === 256 ? '128x128@2x.png' : `${size}x${size}.png`)
        await sharp(svgPath)
            .resize(size, size)
            .png()
            .toFile(outputPath)
        console.log(`已生成: ${path.basename(outputPath)}`)
    }

    // 生成 icon.png (1024x1024)
    await sharp(svgPath)
        .resize(1024, 1024)
        .png()
        .toFile(path.join(iconsDir, 'icon.png'))
    console.log('已生成: icon.png')

    console.log('\n正在生成平台图标 (.icns/.ico)...')
    const npxCmd = process.platform === 'win32' ? 'npx.cmd' : 'npx'
    const result = spawnSync(
        npxCmd,
        ['@tauri-apps/cli', 'icon', path.join(iconsDir, 'icon.png')],
        { stdio: 'inherit' }
    )

    if (result.status !== 0) {
        console.error('tauri icon 执行失败，请检查 @tauri-apps/cli 是否可用。')
        process.exit(result.status ?? 1)
    }

    console.log('\n图标生成完成！')
}

generateIcons().catch(console.error)
