import React, { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';

type NotificationType = 'info' | 'warn' | 'error';

interface NotificationMessage {
  type: NotificationType;
  content: string;
}

interface NotificationProps {
  message: NotificationMessage;
  onClose: () => void;
}

const Notification: React.FC<NotificationProps> = ({ message, onClose }) => {
  const getTypeColor = () => {
    switch (message.type) {
      case 'info': return 'bg-blue-500';
      case 'warn': return 'bg-yellow-500';
      case 'error': return 'bg-red-500';
      default: return 'bg-blue-500';
    }
  };

  const getTypeText = () => {
    switch (message.type) {
      case 'info': return '信息';
      case 'warn': return '警告';
      case 'error': return '错误';
      default: return '信息';
    }
  };

  useEffect(() => {
    const timer = setTimeout(() => {
      onClose();
    }, 5000);

    return () => clearTimeout(timer);
  }, [onClose]);

  return (
    <div className={`p-3 rounded-lg shadow-lg text-white flex items-start backdrop-blur-lg bg-opacity-90 ${getTypeColor()}`}>
      <div className="mr-2 font-medium text-sm">{getTypeText()}:</div>
      <div className="flex-1 text-sm">{message.content}</div>
      <button onClick={onClose} className="ml-3 font-bold hover:bg-white/20 rounded-full w-5 h-5 flex items-center justify-center text-sm">
        ✕
      </button>
    </div>
  );
};

const NotificationContainer: React.FC = () => {
  const [notifications, setNotifications] = useState<{ id: number, message: NotificationMessage }[]>([]);
  const [nextId, setNextId] = useState(1);

  useEffect(() => {
    const unlisten = listen<NotificationMessage>('notify', (event) => {
      const newNotification = {
        id: nextId,
        message: event.payload
      };

      setNotifications(prev => [...prev, newNotification]);
      setNextId(prev => prev + 1);
    });

    return () => {
      unlisten.then(unlistenFn => unlistenFn());
    }
  }, [nextId]);

  const handleClose = (id: number) => {
    setNotifications(prev => prev.filter(notification => notification.id !== id));
  };

  return (
    <div className='flex flex-col gap-2 fixed top-3 right-3 z-50'>
      {notifications.map(({ id, message }) => (
        <Notification
          key={id}
          message={message}
          onClose={() => handleClose(id)}
        />
      ))}
    </div>
  );
};

export default NotificationContainer;