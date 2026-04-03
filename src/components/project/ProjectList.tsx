import { useEffect, useState } from 'react';
import { useProjectStore } from '../../store/projectStore';
import ProjectCard from './ProjectCard';
import { Button } from '../ui/button';
import { Input } from '../ui/input';

interface ProjectListProps {
    onSelectProject: (projectId: string) => void;
    showInput: boolean;
    onShowInput: (show: boolean) => void;
}

export default function ProjectList({ onSelectProject, showInput, onShowInput }: ProjectListProps) {
    const { projects, fetchProjects, createProject, deleteProject } = useProjectStore();
    const [newName, setNewName] = useState('');

    useEffect(() => {
        fetchProjects();
    }, [fetchProjects]);

    const handleCreate = async () => {
        const name = newName.trim();
        if (!name) return;
        await createProject(name);
        setNewName('');
        onShowInput(false);
    };

    const handleDelete = async (id: string) => {
        if (window.confirm('确定要删除此项目吗？所有关联数据将被清除。')) {
            await deleteProject(id);
        }
    };

    return (
        <div className="mx-auto max-w-4xl px-6 py-10">
            <div className="flex items-center justify-between mb-8">
                <h1 className="text-2xl font-bold">VoxFlow 项目</h1>
            </div>

            {showInput && (
                <div className="mb-6 flex gap-3">
                    <Input
                        className="flex-1"
                        placeholder="输入项目名称..."
                        value={newName}
                        onChange={(e) => setNewName(e.target.value)}
                        onKeyDown={(e) => e.key === 'Enter' && handleCreate()}
                        autoFocus
                    />
                    <Button onClick={handleCreate}>创建</Button>
                    <Button variant="outline" onClick={() => { onShowInput(false); setNewName(''); }}>
                        取消
                    </Button>
                </div>
            )}

            {projects.length === 0 ? (
                <p className="text-center text-muted-foreground py-20">暂无项目，点击右上角 + 新建项目</p>
            ) : (
                <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                    {projects.map((p) => (
                        <ProjectCard
                            key={p.id}
                            project={p}
                            onClick={() => onSelectProject(p.id)}
                            onDelete={() => handleDelete(p.id)}
                        />
                    ))}
                </div>
            )}
        </div>
    );
}
