import { useState, useCallback, useEffect } from 'react'
import { open } from '@tauri-apps/plugin-dialog'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { processFiles, type ArchiveResult } from '../lib/api'

export default function ArchiveView() {
  const [isDragging, setIsDragging] = useState(false)
  const [isProcessing, setIsProcessing] = useState(false)
  const [results, setResults] = useState<ArchiveResult[]>([])

  const handleFiles = useCallback(async (filePaths: string[]) => {
    if (filePaths.length === 0) return
    setIsProcessing(true)
    try {
      const res = await processFiles(filePaths)
      setResults(prev => [...res, ...prev].slice(0, 20))
    } catch (e: any) {
      alert('处理失败: ' + e)
    } finally {
      setIsProcessing(false)
    }
  }, [])

  useEffect(() => {
    const unlisten = getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === 'over') {
        setIsDragging(true)
      } else if (event.payload.type === 'drop') {
        setIsDragging(false)
        const paths = event.payload.paths
        const excelPaths = paths.filter(p =>
          p.endsWith('.xlsx') || p.endsWith('.xls') || p.endsWith('.xlsm')
        )
        if (excelPaths.length === 0) {
          alert('未检测到Excel文件，请拖入 .xlsx / .xls / .xlsm 格式的文件')
          return
        }
        if (excelPaths.length < paths.length) {
          alert(`已过滤 ${paths.length - excelPaths.length} 个非Excel文件`)
        }
        handleFiles(excelPaths)
      } else {
        setIsDragging(false)
      }
    })
    return () => { unlisten.then(fn => fn()) }
  }, [handleFiles])

  const handleSelectFiles = async () => {
    try {
      const selected = await open({
        multiple: true,
        filters: [{ name: 'Excel Files', extensions: ['xlsx', 'xls', 'xlsm'] }]
      })
      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected]
        await handleFiles(paths)
      }
    } catch (e: any) {
      console.error('文件选择失败:', e)
    }
  }

  const statusClass = (status: string) => {
    switch (status) {
      case 'success': return 'text-green-600 bg-green-50'
      case 'failed': return 'text-red-600 bg-red-50'
      case 'skipped': return 'text-yellow-600 bg-yellow-50'
      case 'no_rule': return 'text-gray-500 bg-gray-50'
      default: return 'text-gray-500 bg-gray-50'
    }
  }

  const statusLabel = (status: string) => {
    switch (status) {
      case 'success': return '成功'
      case 'failed': return '失败'
      case 'skipped': return '跳过'
      case 'no_rule': return '无规则'
      default: return status
    }
  }

  const truncateFileName = (name: string, maxLen = 12) => {
    if (!name) return '-'
    return name.length <= maxLen ? name : name.substring(0, maxLen) + '...'
  }

  const shortenPath = (path: string) => {
    if (!path) return '-'
    const parts = path.replace(/\\/g, '/').split('/')
    return parts.length >= 2 ? '.../' + parts.slice(-2).join('/') : path
  }

  return (
    <div className="space-y-4">
      <div
        onClick={handleSelectFiles}
        className={`border-2 border-dashed rounded-lg p-10 text-center transition-colors cursor-pointer ${
          isDragging
            ? 'border-blue-400 bg-blue-50'
            : 'border-gray-300 bg-white hover:border-blue-300 hover:bg-gray-50'
        }`}
      >
        <div className="space-y-2">
          <svg className="mx-auto h-10 w-10 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.5"
              d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
          </svg>
          <p className="text-gray-600">将Excel文件拖拽到此处，或点击选择文件</p>
          <p className="text-gray-400 text-sm">支持 .xlsx / .xls / .xlsm 格式</p>
        </div>
      </div>

      {isProcessing && (
        <div className="w-full bg-gray-200 rounded-full h-1.5">
          <div className="bg-blue-600 h-1.5 rounded-full animate-pulse w-3/4" />
        </div>
      )}

      {results.length > 0 && (
        <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 w-28">文件名</th>
                <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 w-24">匹配规则</th>
                <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 w-32">提取日期</th>
                <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 w-48">目标路径</th>
                <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 w-20">状态</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200">
              {results.map((item, index) => (
                <tr key={index} className="h-9">
                  <td className="px-3 py-1.5 text-sm text-gray-900 truncate" title={item.file_name}>{truncateFileName(item.file_name)}</td>
                  <td className="px-3 py-1.5 text-sm text-gray-600 truncate">{item.rule_name || '-'}</td>
                  <td className="px-3 py-1.5 text-sm text-gray-600">{item.date || '-'}</td>
                  <td className="px-3 py-1.5 text-sm text-gray-600 font-mono text-xs truncate" title={item.target_path}>{shortenPath(item.target_path)}</td>
                  <td className="px-3 py-1.5">
                    <span className={`px-1.5 py-0.5 text-xs font-medium rounded ${statusClass(item.status)}`}>
                      {statusLabel(item.status)}
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
