'use client';

import { useState } from 'react';

interface ProjectFormProps {
  onSubmit: (data: ProjectFormData) => Promise<void>;
  onCancel?: () => void;
  initialData?: Partial<ProjectFormData>;
  isLoading?: boolean;
}

export interface ProjectFormData {
  name: string;
  description: string;
  repository_url: string;
  language: string;
}

export default function ProjectForm({ onSubmit, onCancel, initialData, isLoading }: ProjectFormProps) {
  const [formData, setFormData] = useState<ProjectFormData>({
    name: initialData?.name || '',
    description: initialData?.description || '',
    repository_url: initialData?.repository_url || '',
    language: initialData?.language || '',
  });
  const [errors, setErrors] = useState<Partial<Record<keyof ProjectFormData, string>>>({});

  const validate = (): boolean => {
    const newErrors: Partial<Record<keyof ProjectFormData, string>> = {};

    if (!formData.name.trim()) {
      newErrors.name = 'Project name is required';
    }

    if (formData.repository_url && !isValidUrl(formData.repository_url)) {
      newErrors.repository_url = 'Please enter a valid URL';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const isValidUrl = (url: string): boolean => {
    try {
      new URL(url);
      return true;
    } catch {
      return false;
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!validate()) return;

    try {
      await onSubmit(formData);
    } catch (error) {
      console.error('Form submission error:', error);
    }
  };

  const handleChange = (field: keyof ProjectFormData, value: string) => {
    setFormData(prev => ({ ...prev, [field]: value }));
    if (errors[field]) {
      setErrors(prev => ({ ...prev, [field]: undefined }));
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      <div>
        <label htmlFor="name" className="block text-sm font-medium text-gray-700 mb-2">
          Project Name *
        </label>
        <input
          type="text"
          id="name"
          value={formData.name}
          onChange={(e) => handleChange('name', e.target.value)}
          className={`w-full px-4 py-2 border ${errors.name ? 'border-red-500' : 'border-gray-300'} focus:outline-none focus:ring-2 focus:ring-gray-900`}
          disabled={isLoading}
        />
        {errors.name && <p className="mt-1 text-sm text-red-600">{errors.name}</p>}
      </div>

      <div>
        <label htmlFor="description" className="block text-sm font-medium text-gray-700 mb-2">
          Description
        </label>
        <textarea
          id="description"
          value={formData.description}
          onChange={(e) => handleChange('description', e.target.value)}
          rows={4}
          className="w-full px-4 py-2 border border-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-900"
          disabled={isLoading}
        />
      </div>

      <div>
        <label htmlFor="repository_url" className="block text-sm font-medium text-gray-700 mb-2">
          Repository URL
        </label>
        <input
          type="text"
          id="repository_url"
          value={formData.repository_url}
          onChange={(e) => handleChange('repository_url', e.target.value)}
          placeholder="https://github.com/username/repo"
          className={`w-full px-4 py-2 border ${errors.repository_url ? 'border-red-500' : 'border-gray-300'} focus:outline-none focus:ring-2 focus:ring-gray-900`}
          disabled={isLoading}
        />
        {errors.repository_url && <p className="mt-1 text-sm text-red-600">{errors.repository_url}</p>}
      </div>

      <div>
        <label htmlFor="language" className="block text-sm font-medium text-gray-700 mb-2">
          Primary Language
        </label>
        <select
          id="language"
          value={formData.language}
          onChange={(e) => handleChange('language', e.target.value)}
          className="w-full px-4 py-2 border border-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-900"
          disabled={isLoading}
        >
          <option value="">Select a language</option>
          <option value="JavaScript">JavaScript</option>
          <option value="TypeScript">TypeScript</option>
          <option value="Python">Python</option>
          <option value="Rust">Rust</option>
          <option value="Go">Go</option>
          <option value="Java">Java</option>
          <option value="C++">C++</option>
          <option value="Other">Other</option>
        </select>
      </div>

      <div className="flex gap-4 pt-4">
        <button
          type="submit"
          disabled={isLoading}
          className="relative group overflow-hidden bg-gray-900 text-white px-6 py-3 text-sm font-medium transition-all hover:shadow-lg disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <span className="relative z-10">{isLoading ? 'Saving...' : 'Save Project'}</span>
          <div className="absolute inset-0 bg-gray-800 transform scale-x-0 group-hover:scale-x-100 transition-transform duration-300 ease-out origin-left" />
        </button>
        {onCancel && (
          <button
            type="button"
            onClick={onCancel}
            disabled={isLoading}
            className="relative group overflow-hidden border border-gray-900 text-gray-900 px-6 py-3 text-sm font-medium transition-all hover:text-white disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <span className="relative z-10">Cancel</span>
            <div className="absolute inset-0 bg-gray-900 transform translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out" />
          </button>
        )}
      </div>
    </form>
  );
}
