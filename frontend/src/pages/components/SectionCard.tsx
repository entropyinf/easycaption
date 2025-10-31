import React from 'react';

interface SectionCardProps {
    title: string;
    children: React.ReactNode;
    actions?: React.ReactNode;
}

const SectionCard: React.FC<SectionCardProps> = ({ title, children, actions }) => {
    return (
        <div className="bg-white rounded-lg shadow p-6">
            <div className="flex items-center justify-between mb-4">
                <h2 className="text-xl font-semibold">{title}</h2>
                {actions}
            </div>
            <div className="space-y-4">
                {children}
            </div>
        </div>
    );
};

export default SectionCard;