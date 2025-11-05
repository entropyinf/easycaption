import React from 'react';

interface SelectOption {
    value: string;
    label: string;
}

interface SelectInputProps {
    label: string;
    value: string;
    description?: string;
    onChange: (value: string) => void;
    options: SelectOption[];
}

const SelectInput: React.FC<SelectInputProps> = ({
    label,
    value,
    description,
    onChange,
    options
}) => {
    return (
        <div>
            <div>
                <label className="block text-sm font-medium mb-1 text-gray-900">{label}</label>
                {description && <p className="text-xs text-gray-500 mb-1">{description}</p>}
            </div>
            <div className="relative">
                <select
                    value={value}
                    onChange={(e) => onChange(e.target.value)}
                    className="w-full px-2.5 py-1.5 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-transparent appearance-none shadow-sm transition duration-200 ease-in-out"
                >
                    {options.map((option, index) => (
                        <option
                            key={index}
                            value={option.value}
                        >
                            {option.label}
                        </option>
                    ))}
                </select>
                <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 text-gray-700">
                    <svg className="fill-current h-3 w-3" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20">
                        <path d="M9.293 12.95l.707.707L15.657 8l-1.414-1.414L10 10.828 5.757 6.586 4.343 8z"/>
                    </svg>
                </div>
            </div>
        </div>
    );
};

export default SelectInput;