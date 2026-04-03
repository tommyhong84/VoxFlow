import { useEffect, useState } from 'react';
import { Settings, Plus } from 'lucide-react';
import { useProjectStore } from './store/projectStore';
import { useCharacterStore } from './store/characterStore';
import { useScriptStore } from './store/scriptStore';
import ProjectList from './components/project/ProjectList';
import CharacterPanel from './components/character/CharacterPanel';
import ScriptEditor from './components/editor/ScriptEditor';
import SettingsDialog from './components/settings/SettingsDialog';
import ExportPanel from './components/editor/ExportPanel';
import { Button } from './components/ui/button';
import { Tabs, TabsList, TabsTrigger } from './components/ui/tabs';
import {
    Tooltip,
    TooltipContent,
    TooltipProvider,
    TooltipTrigger,
} from './components/ui/tooltip';
import './App.css';

type Tab = 'editor' | 'characters' | 'export';

const tabLabels: Record<Tab, string> = {
    editor: '剧本编辑',
    characters: '角色管理',
    export: '导出',
};

function App() {
    const { currentProject, loadProject } = useProjectStore();
    const [settingsOpen, setSettingsOpen] = useState(false);
    const [activeTab, setActiveTab] = useState<Tab>('editor');
    const [showNewProject, setShowNewProject] = useState(false);
    const { isDirty } = useScriptStore();

    const handleSelectProject = async (projectId: string) => {
        await loadProject(projectId);
    };

    const handleBack = () => {
        if (isDirty && !window.confirm('有未保存的更改，确定要离开吗？')) {
            return;
        }
        useProjectStore.setState({ currentProject: null });
        useCharacterStore.setState({ characters: [] });
        useScriptStore.setState({ lines: [], isDirty: false, streamingText: '' });
        setActiveTab('editor');
    };

    const projectId = currentProject?.project.id;
    useEffect(() => {
        if (currentProject && projectId) {
            useCharacterStore.getState().fetchCharacters();
            useScriptStore.setState({
                lines: currentProject.script_lines,
                isDirty: false,
            });
        }
        // Only re-run when the project ID changes, not on every currentProject update
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [projectId]);

    if (!currentProject) {
        return (
            <TooltipProvider>
                <div className="min-h-screen">
                    <div className="fixed top-4 right-4 z-10 flex items-center gap-1">
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    onClick={() => setShowNewProject(true)}
                                    aria-label="新建项目"
                                >
                                    <Plus className="h-5 w-5" />
                                </Button>
                            </TooltipTrigger>
                            <TooltipContent>新建项目</TooltipContent>
                        </Tooltip>
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    onClick={() => setSettingsOpen(true)}
                                    aria-label="设置"
                                >
                                    <Settings className="h-5 w-5" />
                                </Button>
                            </TooltipTrigger>
                            <TooltipContent>设置</TooltipContent>
                        </Tooltip>
                    </div>
                    <ProjectList
                        onSelectProject={handleSelectProject}
                        showInput={showNewProject}
                        onShowInput={setShowNewProject}
                    />
                    {settingsOpen && <SettingsDialog onClose={() => setSettingsOpen(false)} />}
                </div>
            </TooltipProvider>
        );
    }

    return (
        <TooltipProvider>
            <div className="min-h-screen flex flex-col">
                {/* Header */}
                <header className="flex items-center justify-between border-b border-gray-200 dark:border-gray-700 px-6 py-3">
                    <div className="flex items-center gap-4">
                        <Button variant="link" size="sm" onClick={handleBack}>
                            ← 返回项目列表
                        </Button>
                        <h1 className="text-lg font-semibold">{currentProject.project.name}</h1>
                    </div>
                    <div className="flex items-center gap-2">
                        <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as Tab)}>
                            <TabsList>
                                {(Object.keys(tabLabels) as Tab[]).map((tab) => (
                                    <TabsTrigger key={tab} value={tab}>
                                        {tabLabels[tab]}
                                    </TabsTrigger>
                                ))}
                            </TabsList>
                        </Tabs>
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    onClick={() => setSettingsOpen(true)}
                                    aria-label="设置"
                                >
                                    <Settings className="h-5 w-5" />
                                </Button>
                            </TooltipTrigger>
                            <TooltipContent>设置</TooltipContent>
                        </Tooltip>
                    </div>
                </header>

                {/* Content */}
                <main className="flex-1 overflow-auto">
                    {activeTab === 'editor' && <ScriptEditor />}
                    {activeTab === 'characters' && <CharacterPanel />}
                    {activeTab === 'export' && <ExportPanel />}
                </main>

                {settingsOpen && <SettingsDialog onClose={() => setSettingsOpen(false)} />}
            </div>
        </TooltipProvider>
    );
}

export default App;
