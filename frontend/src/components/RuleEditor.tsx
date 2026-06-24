import { useState, useEffect } from 'react'
import { open } from '@tauri-apps/plugin-dialog'
import { addRule, updateRule, loadExcelHeaders, type Rule } from '../lib/api'

interface Props {
  rule: Rule | null
  onClose: () => void
  onSaved: () => void
}

export default function RuleEditor({ rule, onClose, onSaved }: Props) {
  const [form, setForm] = useState<Rule>({
    id: '',
    name: '',
    feature_columns: [],
    feature_row: 1,
    date_column: '',
    force_precision: 'auto',
    naming_pattern: '',
    auto_archive: true,
  })
  const [headers, setHeaders] = useState<string[]>([])
  const [isLoadingHeaders, setIsLoadingHeaders] = useState(false)
  const [templateFile, setTemplateFile] = useState('')

  useEffect(() => {
    if (rule) {
      setForm({ ...rule })
      if (rule.feature_columns && rule.feature_columns.length > 0) {
        setHeaders([...rule.feature_columns])
        setTemplateFile('已加载')
      }
    }
  }, [rule])

  useEffect(() => {
    if (form.name && !templateFile) {
      setForm(prev => ({
        ...prev,
        naming_pattern: prev.name ? prev.name + '_yyyyMMdd' : '',
      }))
    }
  }, [form.name])

  const isAllSelected = headers.length > 0 && form.feature_columns.length === headers.length

  async function handleLoadTemplate() {
    const selected = await open({
      filters: [{ name: 'Excel Files', extensions: ['xlsx', 'xls', 'xlsm'] }]
    })
    if (!selected) return

    const path = typeof selected === 'string' ? selected : selected as string

    setIsLoadingHeaders(true)
    try {
      const result = await loadExcelHeaders(path, form.feature_row)
      setHeaders(result.headers || [])
      setForm(prev => ({ ...prev, feature_row: result.detected_row || 1 }))
      setTemplateFile(path.split('/').pop() || path)
    } catch (e) {
      alert('加载模板失败: ' + e)
    } finally {
      setIsLoadingHeaders(false)
    }
  }

  function toggleFeatureColumn(col: string) {
    setForm(prev => {
      const idx = prev.feature_columns.indexOf(col)
      const newCols = [...prev.feature_columns]
      if (idx >= 0) {
        newCols.splice(idx, 1)
      } else {
        newCols.push(col)
      }
      return { ...prev, feature_columns: newCols }
    })
  }

  function toggleSelectAll() {
    setForm(prev => ({
      ...prev,
      feature_columns: isAllSelected ? [] : [...headers],
    }))
  }

  function handleSave() {
    if (!form.name.trim()) {
      alert('请输入规则名称')
      return
    }
    if (form.feature_columns.length === 0) {
      alert('请至少选择一个特征列')
      return
    }
    if (!form.naming_pattern.trim()) {
      alert('请输入命名格式')
      return
    }
    if (form.id) {
      updateRule(form.id, form).then(() => onSaved())
    } else {
      addRule(form).then(() => onSaved())
    }
  }

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={onClose}>
      <div className="bg-white rounded-lg shadow-xl w-full max-w-2xl max-h-[90vh] overflow-y-auto" onClick={e => e.stopPropagation()}>
        <div className="p-4 space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-base font-medium text-gray-900">{form.id ? '编辑规则' : '新增规则'}</h3>
            <button onClick={onClose} className="text-gray-400 hover:text-gray-600 text-xl">&times;</button>
          </div>

          <div className="space-y-3">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">规则名称</label>
              <input value={form.name} onChange={e => setForm({ ...form, name: e.target.value })}
                type="text" placeholder="输入规则名称"
                className="w-full px-3 py-1.5 border border-gray-300 rounded text-sm focus:ring-1 focus:ring-blue-500 focus:border-blue-500" />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">模板文件</label>
              <div className="flex items-center space-x-2">
                <button onClick={handleLoadTemplate} disabled={isLoadingHeaders}
                  className="px-3 py-1.5 bg-gray-100 text-gray-700 text-sm font-medium rounded hover:bg-gray-200 disabled:opacity-50 transition-colors">
                  {isLoadingHeaders ? '加载中...' : '选择模板Excel'}
                </button>
                {templateFile && <span className="text-sm text-gray-500 truncate max-w-xs">{templateFile}</span>}
              </div>
            </div>

            {headers.length > 0 && (
              <>
                <div>
                  <div className="flex items-center justify-between mb-1">
                    <label className="text-sm font-medium text-gray-700">特征列</label>
                    <label className="flex items-center space-x-1 cursor-pointer select-none">
                      <input type="checkbox" checked={isAllSelected} onChange={toggleSelectAll}
                        className="rounded border-gray-300 text-blue-600 focus:ring-blue-500" />
                      <span className="text-sm text-gray-600">全选</span>
                    </label>
                  </div>
                  <div className="grid grid-cols-3 gap-1 max-h-40 overflow-y-auto p-2 bg-gray-50 rounded">
                    {headers.map(col => (
                      <label key={col} className="flex items-center space-x-1 text-sm cursor-pointer">
                        <input type="checkbox" checked={form.feature_columns.includes(col)}
                          onChange={() => toggleFeatureColumn(col)}
                          className="rounded border-gray-300 text-blue-600 focus:ring-blue-500" />
                        <span className="text-gray-700 truncate">{col}</span>
                      </label>
                    ))}
                  </div>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">日期列</label>
                  <select value={form.date_column} onChange={e => setForm({ ...form, date_column: e.target.value })}
                    className="w-full px-3 py-1.5 border border-gray-300 rounded text-sm focus:ring-1 focus:ring-blue-500 focus:border-blue-500">
                    <option value="">无</option>
                    {headers.map(col => <option key={col} value={col}>{col}</option>)}
                  </select>
                </div>
              </>
            )}

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">日期精度</label>
              <select value={form.force_precision} onChange={e => setForm({ ...form, force_precision: e.target.value })}
                className="w-full px-3 py-1.5 border border-gray-300 rounded text-sm focus:ring-1 focus:ring-blue-500 focus:border-blue-500">
                <option value="auto">自动判断</option>
                <option value="day">日精度</option>
                <option value="month">月精度</option>
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">命名格式</label>
              <input value={form.naming_pattern} onChange={e => setForm({ ...form, naming_pattern: e.target.value })}
                type="text" placeholder="如：商家明细_yyyyMMdd"
                className="w-full px-3 py-1.5 border border-gray-300 rounded text-sm focus:ring-1 focus:ring-blue-500 focus:border-blue-500" />
              <p className="mt-0.5 text-xs text-gray-400">支持变量：yyyy(年) MM(月) dd(日)</p>
            </div>

            <div className="flex items-center space-x-2">
              <input type="checkbox" checked={form.auto_archive} onChange={e => setForm({ ...form, auto_archive: e.target.checked })}
                className="rounded border-gray-300 text-blue-600 focus:ring-blue-500" />
              <label className="text-sm text-gray-700">启用自动转存</label>
            </div>
          </div>

          <div className="flex justify-end space-x-3 pt-3 border-t border-gray-200">
            <button onClick={onClose} className="px-4 py-1.5 text-sm font-medium text-gray-700 bg-gray-100 rounded hover:bg-gray-200 transition-colors">
              取消
            </button>
            <button onClick={handleSave} className="px-4 py-1.5 text-sm font-medium text-white bg-blue-600 rounded hover:bg-blue-700 transition-colors">
              保存
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}
