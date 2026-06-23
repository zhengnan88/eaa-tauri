import { useState, useEffect } from 'react'
import { getDataReadyStatus, scanTodayMonitors, type DataReadyStatus } from '../lib/api'

export default function ReadyStatus() {
  const [statuses, setStatuses] = useState<DataReadyStatus[]>([])
  const [selectedDate, setSelectedDate] = useState('')
  const [isScanningToday, setIsScanningToday] = useState(false)
  const today = new Date()

  useEffect(() => {
    const yesterday = new Date()
    yesterday.setDate(yesterday.getDate() - 1)
    setSelectedDate(yesterday.toISOString().split('T')[0])
  }, [])

  useEffect(() => {
    if (selectedDate) loadData()
  }, [selectedDate])

  async function loadData() {
    if (!selectedDate) return
    setStatuses(await getDataReadyStatus(selectedDate))
  }

  async function handleScanToday() {
    setIsScanningToday(true)
    try {
      const result = await scanTodayMonitors()
      const msg = result.scanned_files > 0
        ? `扫描完成，发现 ${result.scanned_files} 个今日文件`
        : '扫描完成，未发现今日文件'
      alert(msg)
      await loadData()
    } catch (e: any) {
      alert('扫描出错：' + e)
    } finally {
      setIsScanningToday(false)
    }
  }

  const readyCount = statuses.filter(s => s.is_ready).length
  const totalCount = statuses.length

  const minDate = new Date(today)
  minDate.setDate(minDate.getDate() - 30)
  const minDateStr = minDate.toISOString().split('T')[0]
  const maxDateStr = today.toISOString().split('T')[0]

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <h2 className="text-base font-medium text-gray-900">数据就绪情况</h2>
        <span className="text-sm text-gray-600">就绪: {readyCount}/{totalCount}</span>
      </div>

      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-2">
          <label className="text-sm text-gray-600">查询日期:</label>
          <input type="date" value={selectedDate} onChange={e => setSelectedDate(e.target.value)}
            min={minDateStr} max={maxDateStr}
            className="px-3 py-1.5 border border-gray-300 rounded text-sm focus:ring-1 focus:ring-blue-500 focus:border-blue-500" />
        </div>
        <button onClick={handleScanToday} disabled={isScanningToday}
          className="px-3 py-1.5 bg-teal-600 text-white text-sm font-medium rounded hover:bg-teal-700 disabled:opacity-50 transition-colors">
          {isScanningToday ? '扫描中...' : '一键扫描（今日）'}
        </button>
      </div>

      {statuses.length === 0 ? (
        <div className="text-center py-12 text-gray-500">暂无规则配置</div>
      ) : (
        <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 w-48">规则名称</th>
                <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 w-32">日期</th>
                <th className="px-4 py-2 text-left text-xs font-medium text-gray-500">文件名</th>
                <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 w-20">就绪</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200">
              {statuses.map((item, index) => (
                <tr key={index}>
                  <td className="px-4 py-2 text-sm text-gray-900">{item.rule_name}</td>
                  <td className="px-4 py-2 text-sm text-gray-600">{item.date}</td>
                  <td className="px-4 py-2 text-sm text-gray-600">{item.file_name || '-'}</td>
                  <td className="px-4 py-2 text-center">
                    <span className={item.is_ready ? 'text-green-600' : 'text-red-500'}>
                      {item.is_ready ? '✓' : '✗'}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}
