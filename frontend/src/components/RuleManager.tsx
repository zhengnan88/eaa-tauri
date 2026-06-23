import { useState, useEffect } from 'react'
import { getRules, deleteRule, moveRule, scanTodayMonitors, type Rule } from '../lib/api'
import RuleEditor from './RuleEditor'

export default function RuleManager() {
  const [rules, setRules] = useState<Rule[]>([])
  const [showEditor, setShowEditor] = useState(false)
  const [editingRule, setEditingRule] = useState<Rule | null>(null)
  const [isScanningToday, setIsScanningToday] = useState(false)

  useEffect(() => { loadRules() }, [])

  async function loadRules() {
    setRules(await getRules())
  }

  function handleAdd() {
    setEditingRule(null)
    setShowEditor(true)
  }

  function handleEdit(rule: Rule) {
    setEditingRule(rule)
    setShowEditor(true)
  }

  async function handleDelete(id: string) {
    if (!confirm('确定要删除此规则吗？')) return
    await deleteRule(id)
    await loadRules()
  }

  async function handleMove(id: string, dir: string) {
    await moveRule(id, dir)
    await loadRules()
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

  function truncateText(text: string, maxChars = 20) {
    if (!text) return ''
    return text.length <= maxChars ? text : text.substring(0, maxChars) + '...'
  }

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <h2 className="text-base font-medium text-gray-900">规则管理</h2>
        <div className="flex items-center space-x-2">
          <button onClick={handleScanToday} disabled={isScanningToday}
            className="px-3 py-1.5 bg-teal-600 text-white text-sm font-medium rounded hover:bg-teal-700 disabled:opacity-50 transition-colors">
            {isScanningToday ? '扫描中...' : '一键扫描（今日）'}
          </button>
          <button onClick={handleAdd} className="px-3 py-1.5 bg-blue-600 text-white text-sm font-medium rounded hover:bg-blue-700 transition-colors">
            新增规则
          </button>
        </div>
      </div>

      {rules.length === 0 ? (
        <div className="text-center py-12 text-gray-500">暂无规则，请点击"新增规则"创建</div>
      ) : (
        <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 w-10">#</th>
                <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 w-28">规则名称</th>
                <th className="px-3 py-2 text-left text-xs font-medium text-gray-500">特征列</th>
                <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 w-14">特征行</th>
                <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 w-14">日期列</th>
                <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 w-40">命名格式</th>
                <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 w-20">转存</th>
                <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 w-32">操作</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200">
              {rules.map((rule, index) => (
                <tr key={rule.id} className="h-9">
                  <td className="px-3 py-1.5 text-sm text-gray-500">{index + 1}</td>
                  <td className="px-3 py-1.5 text-sm font-medium text-gray-900 truncate">{rule.name}</td>
                  <td className="px-3 py-1.5 text-sm text-gray-600 truncate" title={(rule.feature_columns || []).join(', ')}>
                    {truncateText((rule.feature_columns || []).join(', '))}
                  </td>
                  <td className="px-3 py-1.5 text-sm text-gray-600">{rule.feature_row || 1}</td>
                  <td className="px-3 py-1.5 text-sm text-gray-600 truncate">{rule.date_column || '无'}</td>
                  <td className="px-3 py-1.5 text-sm text-gray-600 font-mono text-xs truncate">{rule.naming_pattern}</td>
                  <td className="px-3 py-1.5">
                    <span className={rule.auto_archive ? 'text-green-600 text-sm whitespace-nowrap' : 'text-gray-400 text-sm whitespace-nowrap'}>
                      {rule.auto_archive ? '已启用' : '未启用'}
                    </span>
                  </td>
                  <td className="px-3 py-1.5">
                    <div className="flex items-center space-x-1 whitespace-nowrap">
                      <button onClick={() => handleMove(rule.id, 'up')} disabled={index === 0}
                        className="p-1 text-gray-400 hover:text-gray-600 disabled:opacity-30 text-xs">↑</button>
                      <button onClick={() => handleMove(rule.id, 'down')} disabled={index === rules.length - 1}
                        className="p-1 text-gray-400 hover:text-gray-600 disabled:opacity-30 text-xs">↓</button>
                      <button onClick={() => handleEdit(rule)} className="px-2 py-0.5 text-xs text-blue-600 hover:text-blue-800 whitespace-nowrap">编辑</button>
                      <button onClick={() => handleDelete(rule.id)} className="px-2 py-0.5 text-xs text-red-600 hover:text-red-800 whitespace-nowrap">删除</button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {showEditor && (
        <RuleEditor rule={editingRule} onClose={() => setShowEditor(false)} onSaved={() => { setShowEditor(false); loadRules() }} />
      )}
    </div>
  )
}
