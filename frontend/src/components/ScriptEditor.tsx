import { useState, useEffect } from 'react'
import { open } from '@tauri-apps/plugin-dialog'
import { type Script } from '../lib/api'

interface Props {
  script: Script | null
  onSave: (data: Script) => void
  onClose: () => void
}

export default function ScriptEditor({ script, onSave, onClose }: Props) {
  const [form, setForm] = useState<Script>({
    id: '',
    name: '',
    script_path: '',
    param_type: 'both',
    order: 0,
  })

  useEffect(() => {
    if (script) {
      setForm({ ...script })
    }
  }, [script])

  async function handleSelectFile() {
    const selected = await open({
      filters: [{ name: 'Python Scripts', extensions: ['py'] }]
    })
    if (selected) {
      const path = typeof selected === 'string' ? selected : selected as string
      setForm({ ...form, script_path: path.split('/').pop() || path })
    }
  }

  function handleSave() {
    if (!form.name.trim()) {
      alert('请输入脚本名称')
      return
    }
    if (!form.script_path.trim()) {
      alert('请选择脚本文件')
      return
    }
    onSave({ ...form })
  }

  return (
    <div className="fixed inset-0 bg-black/40 flex items-center justify-center z-50" onClick={onClose}>
      <div className="bg-white rounded-lg shadow-xl w-full max-w-md mx-4" onClick={e => e.stopPropagation()}>
        <div className="flex items-center justify-between p-4 border-b border-gray-100">
          <h3 className="text-base font-medium text-gray-900">{script ? '编辑脚本' : '新增脚本'}</h3>
          <button onClick={onClose} className="text-gray-400 hover:text-gray-600 text-xl">&times;</button>
        </div>

        <div className="p-4 space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">脚本名称</label>
            <input value={form.name} onChange={e => setForm({ ...form, name: e.target.value })}
              type="text" placeholder="如：盈亏报表"
              className="w-full px-3 py-1.5 border border-gray-300 rounded text-sm focus:ring-1 focus:ring-blue-500 focus:border-blue-500" />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">脚本文件</label>
            <div className="flex items-center space-x-2">
              <button onClick={handleSelectFile}
                className="px-3 py-1.5 bg-gray-100 text-gray-700 text-sm font-medium rounded hover:bg-gray-200 transition-colors">
                选择脚本
              </button>
              <span className="text-sm text-gray-500 truncate">{form.script_path || '未选择'}</span>
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">参数类型</label>
            <select value={form.param_type} onChange={e => setForm({ ...form, param_type: e.target.value })}
              className="w-full px-3 py-1.5 border border-gray-300 rounded text-sm focus:ring-1 focus:ring-blue-500 focus:border-blue-500 bg-white">
              <option value="day">日报（8位日期）</option>
              <option value="month">月报（6位年月）</option>
              <option value="both">日报/月报皆可</option>
            </select>
          </div>
        </div>

        <div className="flex items-center justify-end space-x-3 p-4 border-t border-gray-100">
          <button onClick={onClose} className="px-4 py-1.5 text-sm font-medium text-gray-700 bg-gray-100 rounded hover:bg-gray-200 transition-colors">
            取消
          </button>
          <button onClick={handleSave} className="px-4 py-1.5 text-sm font-medium text-white bg-blue-600 rounded hover:bg-blue-700 transition-colors">
            保存
          </button>
        </div>
      </div>
    </div>
  )
}
