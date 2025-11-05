import React, { useState } from 'react';

interface NumberInputProps {
    label: string;
    value: number;
    onChange: (value: number) => void;
    placeholder?: string;
    step?: string;
}

const NumberInput: React.FC<NumberInputProps> = ({
    label,
    value,
    onChange,
    placeholder = '',
    step = '1'
}) => {
    const [localValue, setLocalValue] = useState<string>(value.toString());

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        setLocalValue(e.target.value);
    };

    const handleBlur = () => {
        const numValue = parseFloat(localValue) || 0;
        if (numValue !== value) {
            onChange(numValue);
        }
    };

    React.useEffect(() => {
        setLocalValue(value.toString());
    }, [value]);

    return (
        <div>
            <label className="block text-sm font-medium mb-1 text-gray-900">{label}</label>
            <input
                type="number"
                step={step}
                value={localValue}
                onChange={handleChange}
                onBlur={handleBlur}
                className="w-full px-2.5 py-1.5 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-transparent transition duration-200 ease-in-out shadow-sm"
                placeholder={placeholder}
            />
        </div>
    );
};

export default NumberInput;