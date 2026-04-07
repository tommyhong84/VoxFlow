import { useState } from 'react';
import { Settings, Plus } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import ProjectList from '../project/ProjectList';
import SettingsDialog from '../settings/SettingsDialog';
import { Button } from '../ui/button';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '../ui/tooltip';
import ToastContainer from './ToastContainer';

interface ProjectListPageProps {
    onSelectProject: (projectId: string) => void;
}

export default function ProjectListPage({ onSelectProject }: ProjectListPageProps) {
    const { t } = useTranslation();
    const [settingsOpen, setSettingsOpen] = useState(false);
    const [showInput, setShowInput] = useState(false);

    return (
        <TooltipProvider>
            <div className="min-h-screen">
                <ToastContainer />
                <div className="fixed top-4 right-4 z-10 flex items-center gap-1">
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <Button
                                variant="ghost"
                                size="icon"
                                onClick={() => setShowInput(true)}
                                aria-label={t('app.newProject')}
                            >
                                <Plus className="h-5 w-5" />
                            </Button>
                        </TooltipTrigger>
                        <TooltipContent>{t('app.newProject')}</TooltipContent>
                    </Tooltip>
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <Button
                                variant="ghost"
                                size="icon"
                                onClick={() => setSettingsOpen(true)}
                                aria-label={t('app.settings')}
                            >
                                <Settings className="h-5 w-5" />
                            </Button>
                        </TooltipTrigger>
                        <TooltipContent>{t('app.settings')}</TooltipContent>
                    </Tooltip>
                </div>
                <ProjectList
                    onSelectProject={onSelectProject}
                    showInput={showInput}
                    onShowInput={setShowInput}
                />
                {settingsOpen && <SettingsDialog onClose={() => setSettingsOpen(false)} />}
            </div>
        </TooltipProvider>
    );
}
