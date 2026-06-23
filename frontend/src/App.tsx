import { useState } from 'react'
import ArchiveView from './components/ArchiveView'
import ScriptRunner from './components/ScriptRunner'
import ScriptManager from './components/ScriptManager'
import RuleManager from './components/RuleManager'
import ReadyStatus from './components/ReadyStatus'
import Settings from './components/Settings'

type TabKey = 'archive' | 'runner' | 'scripts' | 'rules' | 'ready' | 'settings'

const navItems: { key: TabKey; label: string }[] = [
  { key: 'archive', label: 'Excel归档' },
  { key: 'runner', label: '脚本执行' },
  { key: 'scripts', label: '脚本管理' },
  { key: 'rules', label: '规则管理' },
  { key: 'ready', label: '就绪情况' },
  { key: 'settings', label: '设置' },
]

export default function App() {
  const [currentTab, setCurrentTab] = useState<TabKey>('archive')

  return (
    <div className="h-screen flex flex-col bg-gray-50">
      <header className="sticky top-0 z-10 bg-white border-b border-gray-200 shrink-0">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-end h-12">
            <nav className="flex space-x-0.5">
              {navItems.map((item) => (
                <button
                  key={item.key}
                  onClick={() => setCurrentTab(item.key)}
                  className={`px-3 py-1.5 text-sm font-medium transition-colors ${
                    currentTab === item.key
                      ? 'bg-blue-50 text-blue-700'
                      : 'text-gray-500 hover:text-gray-900 hover:bg-gray-100'
                  }`}
                >
                  {item.label}
                </button>
              ))}
            </nav>
          </div>
        </div>
      </header>

      <main className="flex-1 max-w-7xl w-full mx-auto px-4 sm:px-6 lg:px-8 py-4 overflow-y-auto">
        {currentTab === 'archive' && <ArchiveView />}
        {currentTab === 'runner' && <ScriptRunner />}
        {currentTab === 'scripts' && <ScriptManager />}
        {currentTab === 'rules' && <RuleManager />}
        {currentTab === 'ready' && <ReadyStatus />}
        {currentTab === 'settings' && <Settings />}
      </main>
    </div>
  )
}
