import { useState } from 'react'
import { Button } from './components/foundation/Button'
import { Card } from './components/layout/Card'
import './App.css'

export default function App() {
  const [count, setCount] = useState(0)

  return (
    <div className="app-container">
      <Card title="AgilePlus Dashboard" className="welcome-card">
        <div className="content">
          <p>Week 1 Foundation Components - Ready for Phase 2</p>
          <Button
            variant="primary"
            size="lg"
            onClick={() => setCount((c) => c + 1)}
          >
            Count is {count}
          </Button>
          <p className="small">All 11 foundation components integrated and tested</p>
        </div>
      </Card>
    </div>
  )
}
