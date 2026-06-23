import { useState, useEffect } from 'react'
import { getScripts, addScript, updateScript, deleteScript, moveScript, type Script } from '../lib/api'
import ScriptEditor from './ScriptEditor'

export default function ScriptManager() {
  const [scripts, setScripts] = useState<Script[]>([])
  const [showEditor, setShowEditor] = useState(false)
  const [editingScript, setEditingScript] = useState<Script | null>(null)

  useEffect(() => { loadScripts() }, [])

  async function loadScripts() {
    const list = await getScripts()
    setScripts([...list].sort((a, b) => (a.order || 0) - (b.order || 0)))
  }

  function getParamTypeLabel(type: string) {
    if (type === 'day') return '日报'
    if (type === 'month') return '月报'
    return '日报/月报'
  }

  async function handleAdd() {
    setEditingScript(null)
    setShowEditor(true)
  }

  async function handleEdit(script: Script) {
    setEditingScript(script)
    setShowEditor(true)
  }

  async function handleDelete(id: string) {
    if (!confirm('确定要删除此脚本吗？')) return
    await deleteScript(id)
    await loadScripts()
  }

  async function handleMove(id: string, dir: string) {
    await moveScript(id, dir)
    await loadScripts()
  }

  async function handleSave(data: Script) {
    if (data.id) {
      await updateScript(data.id, data)
    } else {
      await addScript(data)
    }
    setShowEditor(false)
    await loadScripts()
  }

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <h2 className="text-base font-medium text-gray-900">脚本管理</h2>
        <button onClick={handleAdd} className="px-3 py-1.5 bg-blue-600 text-white text-sm font-medium rounded hover:bg-blue-700 transition-colors">
          添加脚本
        </button>
      </div>

      {scripts.length === 0 ? (
        <div className="text-center py-12 text-gray-400 text-sm">暂无脚本</div>
      ) : (
        <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 w-10">#</th>
                <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 w-36">脚本名称</th>
                <th className="px-3 py-2 text-left text-xs font-medium text-gray-500">脚本路径</th>
                <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 w-24">参数类型</th>
                <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 w-32">操作</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200">
              {scripts.map((script, index) => (
                <tr key={script.id} className="h-9">
                  <td className="px-3 py-1.5 text-sm text-gray-500">{index + 1}</td>
                  <td className="px-3 py-1.5 text-sm font-medium text-gray-900">{script.name}</td>
                  <td className="px-3 py-1.5 text-sm text-gray-600 font-mono text-xs">{script.script_path}</td>
                  <td className="px-3 py-1.5 text-sm text-gray-600">{getParamTypeLabel(script.param_type)}</td>
                  <td className="px-3 py-1.5">
                    <div className="flex items-center space-x-1 whitespace-nowrap">
                      <button onClick={() => handleMove(script.id, 'up')} disabled={index === 0}
                        className="p-1 text-gray-400 hover:text-gray-600 disabled:opacity-30 text-xs">↑</button>
                      <button onClick={() => handleMove(script.id, 'down')} disabled={index === scripts.length - 1}
                        className="p-1 text-gray-400 hover:text-gray-600 disabled:opacity-30 text-xs">↓</button>
                      <button onClick={() => handleEdit(script)} className="px-2 py-0.5 text-xs text-blue-600 hover:text-blue-800 whitespace-nowrap">编辑</button>
                      <button onClick={() => handleDelete(script.id)} className="px-2 py-0.5 text-xs text-red-600 hover:text-red-800 whitespace-nowrap">删除</button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {showEditor && (
        <ScriptEditor script={editingScript} onSave={handleSave} onClose={() => setShowEditor(false)} />
      )}
    </div>
  )
}
