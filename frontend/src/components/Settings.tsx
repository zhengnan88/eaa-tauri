import { useState, useEffect } from 'react'
import {
  getSettings, saveSettings, getDataCenter, setDataCenter,
  getMonitorFolders, addMonitorFolder, deleteMonitorFolder,
  scanAllMonitors, scanTodayMonitors, pickFolder,
  type AppSettings, type MonitorFolder
} from '../lib/api'

export default function Settings() {
  const [settings, setSettings] = useState<AppSettings>({
    ignore_case: false,
    ignore_space: true,
    log_retention_days: 30,
  })
  const [dataCenter, setDataCenterPath] = useState('')
  const [folders, setFolders] = useState<MonitorFolder[]>([])
  const [isScanning, setIsScanning] = useState(false)
  const [isScanningToday, setIsScanningToday] = useState(false)

  useEffect(() => { loadAll() }, [])

  async function loadAll() {
    const s = await getSettings()
    setSettings(s)
    const dc = await getDataCenter()
    setDataCenterPath(dc)
    setFolders(await getMonitorFolders())
  }

  async function handlePickDataCenter() {
    const selected = await pickFolder()
    if (selected) {
      const result = await setDataCenter(selected)
      if (!result.success) {
        alert(result.message)
        return
      }
      setDataCenterPath(selected)
    }
  }

  async function handleAddFolder() {
    const selected = await pickFolder()
    if (selected) {
      await addMonitorFolder({
        path: selected,
        include_sub: true,
        scan_interval_min: 60,
        last_scan_time: '',
      })
      setFolders(await getMonitorFolders())
    }
  }

  async function handleDeleteFolder(index: number) {
    if (!confirm('确定要删除此监控文件夹吗？')) return
    await deleteMonitorFolder(index)
    setFolders(await getMonitorFolders())
  }

  async function handleScanAll() {
    setIsScanning(true)
    try {
      const result = await scanAllMonitors()
      const msg = result.scanned_files > 0
        ? `扫描完成，共扫描 ${result.scanned_files} 个Excel文件`
        : '扫描完成，未发现Excel文件'
      alert(msg)
      setFolders(await getMonitorFolders())
    } catch (e: any) {
      alert('扫描出错：' + e)
    } finally {
      setIsScanning(false)
    }
  }

  async function handleScanToday() {
    setIsScanningToday(true)
    try {
      const result = await scanTodayMonitors()
      const msg = result.scanned_files > 0
        ? `扫描完成，发现 ${result.scanned_files} 个今日文件`
        : '扫描完成，未发现今日文件'
      alert(msg)
    } catch (e: any) {
      alert('扫描出错：' + e)
    } finally {
      setIsScanningToday(false)
    }
  }

  async function handleSettingsChange(newSettings: AppSettings) {
    setSettings(newSettings)
    await saveSettings(newSettings)
  }

  return (
    <div className="space-y-4">
      <h2 className="text-base font-medium text-gray-900">设置</h2>

      <div className="bg-white rounded-lg border border-gray-200 p-4 space-y-3">
        <h3 className="text-sm font-medium text-gray-900">数据中心</h3>
        <div className="flex items-center space-x-2">
          <input value={dataCenter} type="text" readOnly
            className="flex-1 px-3 py-1.5 border border-gray-300 rounded text-sm bg-gray-50"
            placeholder="数据中心根目录" />
          <button onClick={handlePickDataCenter}
            className="px-3 py-1.5 bg-gray-100 text-gray-700 text-sm font-medium rounded hover:bg-gray-200 transition-colors">
            选择
          </button>
        </div>
      </div>

      <div className="bg-white rounded-lg border border-gray-200 p-4 space-y-3">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-medium text-gray-900">监控文件夹</h3>
          <div className="flex items-center space-x-2">
            <button onClick={handleScanToday} disabled={isScanningToday}
              className="px-2 py-1 bg-teal-600 text-white text-xs font-medium rounded hover:bg-teal-700 disabled:opacity-50 transition-colors">
              {isScanningToday ? '扫描中...' : '一键扫描（今日）'}
            </button>
            <button onClick={handleScanAll} disabled={isScanning}
              className="px-2 py-1 bg-green-600 text-white text-xs font-medium rounded hover:bg-green-700 disabled:opacity-50 transition-colors">
              {isScanning ? '扫描中...' : '全量扫描'}
            </button>
            <button onClick={handleAddFolder}
              className="px-2 py-1 bg-blue-600 text-white text-xs font-medium rounded hover:bg-blue-700 transition-colors">
              添加
            </button>
          </div>
        </div>

        {folders.length === 0 ? (
          <div className="text-center py-4 text-gray-400 text-sm">暂无监控文件夹</div>
        ) : (
          <div className="divide-y divide-gray-100">
            {folders.map((folder, index) => (
              <div key={index} className="flex items-center justify-between py-2">
                <div className="flex-1 min-w-0">
                  <p className="text-sm text-gray-900 truncate">{folder.path}</p>
                  <div className="flex items-center space-x-2 mt-0.5">
                    <span className="text-xs text-gray-500">
                      {folder.include_sub ? '含子文件夹' : '仅当前目录'}
                    </span>
                    <span className="text-xs text-gray-400">
                      最后扫描: {folder.last_scan_time ? folder.last_scan_time.substring(0, 16).replace('T', ' ') : '未扫描'}
                    </span>
                  </div>
                </div>
                <button onClick={() => handleDeleteFolder(index)}
                  className="text-xs text-red-600 hover:text-red-800 ml-2 shrink-0">删除</button>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="bg-white rounded-lg border border-gray-200 p-4 space-y-3">
        <h3 className="text-sm font-medium text-gray-900">高级选项</h3>
        <div className="space-y-2">
          <label className="flex items-center space-x-2 cursor-pointer">
            <input type="checkbox" checked={settings.ignore_case}
              onChange={e => handleSettingsChange({ ...settings, ignore_case: e.target.checked })}
              className="rounded border-gray-300 text-blue-600 focus:ring-blue-500" />
            <span className="text-sm text-gray-700">表头匹配时忽略大小写</span>
          </label>
          <label className="flex items-center space-x-2 cursor-pointer">
            <input type="checkbox" checked={settings.ignore_space}
              onChange={e => handleSettingsChange({ ...settings, ignore_space: e.target.checked })}
              className="rounded border-gray-300 text-blue-600 focus:ring-blue-500" />
            <span className="text-sm text-gray-700">自动去除列名首尾空格</span>
          </label>
          <div className="flex items-center space-x-2">
            <label className="text-sm text-gray-700">日志保留天数</label>
            <input value={settings.log_retention_days} type="number" min="1"
              onChange={e => handleSettingsChange({ ...settings, log_retention_days: Number(e.target.value) })}
              className="w-16 px-2 py-1 border border-gray-300 rounded text-sm focus:ring-1 focus:ring-blue-500" />
          </div>
        </div>
      </div>
    </div>
  )
}
